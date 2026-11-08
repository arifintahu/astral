// The GitHub release body: this tag's CHANGELOG.md section plus the pull requests and contributors
// since the previous stable tag.
//   node scripts/release-notes.mjs vX.Y.Z [head-ref]
// `head-ref` (default: the tag) is what the pull requests are counted up to, to try the notes before tagging.
//
// CHANGELOG.md `### Highlights` -> Highlights; `Added`/`Changed`/`Removed` -> Changes;
// `Fixed`/`Security` -> Fixes. Pull requests and contributors come from the GitHub API (needs
// GITHUB_TOKEN and GITHUB_REPOSITORY, and git tags); without a token they are left out.
// A stable tag must have a Highlights section. A pre-release tag (vX.Y.Z-rc.1) without its own
// changelog section gets a placeholder, so release workflow tests need no changelog entry.
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const GROUPS = {
  Highlights: ["Highlights"],
  Changes: ["Added", "Changed", "Removed"],
  Fixes: ["Fixed", "Security"],
};

const semver = (tag) => tag.replace(/^v/, "").split("-")[0].split(".").map(Number);
const compare = (a, b) => {
  const [x, y] = [semver(a), semver(b)];
  return x[0] - y[0] || x[1] - y[1] || x[2] - y[2];
};

/** Newest stable tag below `tag`; the notes cover everything since it. */
export function previousStableTag(tags, tag) {
  return tags
    .filter((t) => /^v\d+\.\d+\.\d+$/.test(t) && t !== tag && compare(t, tag) < 0)
    .sort(compare)
    .pop();
}

/** Splits the text under a `## [version]` heading into its intro and `### ` subsections. */
export function parseChangelog(changelog, version) {
  const lines = changelog.replace(/\r\n/g, "\n").split("\n");
  const start = lines.findIndex((l) => l.startsWith(`## [${version}]`));
  if (start < 0) return null;
  const end = lines.findIndex((l, i) => i > start && l.startsWith("## "));
  const parsed = { intro: [], sections: {} };
  let current = null;
  for (const line of lines.slice(start + 1, end < 0 ? undefined : end)) {
    const heading = line.match(/^### (.+)/);
    if (heading) {
      current = heading[1].trim();
      parsed.sections[current] = [];
    } else {
      (current ? parsed.sections[current] : parsed.intro).push(line);
    }
  }
  const text = (ls) => ls.join("\n").trim();
  return {
    intro: text(parsed.intro),
    sections: Object.fromEntries(Object.entries(parsed.sections).map(([k, v]) => [k, text(v)])),
  };
}

/** PR numbers from merge commits ("Merge pull request #12 ...") and squash commits ("... (#12)"). */
export function pullNumbers(messages) {
  const found = new Set();
  for (const message of messages) {
    const subject = message.split("\n")[0];
    const n = subject.match(/^Merge pull request #(\d+)/)?.[1] ?? subject.match(/\(#(\d+)\)$/)?.[1];
    if (n) found.add(Number(n));
  }
  return [...found].sort((a, b) => a - b);
}

const isBot = (login) => login.endsWith("[bot]");
// A zero-width space after "@" keeps a PR title from pinging whoever it names.
const plain = (title) => title.replace(/@(?=\w)/g, "@​");

export function compose({ tag, parsed, prs, previousTag, repo }) {
  const version = tag.replace(/^v/, "");
  const out = [`## ${version}`];
  if (parsed?.intro) out.push(parsed.intro);
  for (const [heading, names] of Object.entries(GROUPS)) {
    const body = names.map((n) => parsed?.sections[n]).filter(Boolean).join("\n");
    if (body) out.push(`### ${heading}\n\n${body}`);
  }
  if (!parsed) out.push(`Pre-release ${tag} for testing the release workflow.`);
  if (prs.length) {
    const lines = prs.map((p) => {
      const thanks = isBot(p.author) ? "" : ` Thanks @${p.author}.`;
      return `- **PR #${p.number}** ${plain(p.title)}.${thanks}`;
    });
    out.push(`### Pull requests\n\n${lines.join("\n")}`);
    const counts = new Map();
    for (const { author } of prs) if (!isBot(author)) counts.set(author, (counts.get(author) ?? 0) + 1);
    const people = [...counts].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0])).map(([a]) => `@${a}`);
    if (people.length) out.push(`### Contributors\n\nThanks ${people.join(", ")}.`);
  }
  if (previousTag && repo) {
    out.push(`**Full changelog**: https://github.com/${repo}/compare/${previousTag}...${tag}`);
  }
  return out.join("\n\n");
}

async function github(repo, token, path) {
  const res = await fetch(`https://api.github.com/repos/${repo}/${path}`, {
    headers: {
      authorization: `Bearer ${token}`,
      accept: "application/vnd.github+json",
      "x-github-api-version": "2022-11-28",
    },
  });
  if (!res.ok) throw new Error(`GitHub API ${res.status} for ${path}`);
  return res.json();
}

async function pullRequests({ repo, token, previousTag, tag }) {
  const messages = [];
  for (let page = 1; ; page++) {
    const range = `compare/${previousTag}...${tag}?per_page=100&page=${page}`;
    const { commits, total_commits: total } = await github(repo, token, range);
    messages.push(...commits.map((c) => c.commit.message));
    if (!commits.length || messages.length >= total) break;
  }
  const pulls = await Promise.all(pullNumbers(messages).map((n) => github(repo, token, `pulls/${n}`)));
  return pulls.map((p) => ({ number: p.number, title: p.title, author: p.user.login }));
}

async function main(tag, head = tag) {
  const version = tag.replace(/^v/, "");
  const parsed = parseChangelog(readFileSync("CHANGELOG.md", "utf8"), version);
  const stable = !version.includes("-");
  if (stable && !parsed?.sections.Highlights) {
    throw new Error(`CHANGELOG.md needs a "## [${version}]" section with a "### Highlights" list`);
  }
  const { GITHUB_TOKEN: token, GITHUB_REPOSITORY: repo } = process.env;
  const tags = execFileSync("git", ["tag", "--list", "v*"], { encoding: "utf8" }).split("\n").filter(Boolean);
  const previousTag = previousStableTag(tags, tag);
  let prs = [];
  if (token && repo && previousTag) {
    prs = await pullRequests({ repo, token, previousTag, tag: head });
  } else {
    console.error("No GITHUB_TOKEN, GITHUB_REPOSITORY or previous tag: leaving out pull requests and contributors");
  }
  console.log(compose({ tag, parsed, prs, previousTag, repo }));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const tag = process.argv[2];
  if (!tag) {
    console.error("usage: node scripts/release-notes.mjs vX.Y.Z");
    process.exit(1);
  }
  main(tag, process.argv[3]).catch((e) => {
    console.error(e.message);
    process.exit(1);
  });
}
