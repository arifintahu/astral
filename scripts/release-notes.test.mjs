import { test } from "node:test";
import assert from "node:assert/strict";
import { compose, parseChangelog, previousStableTag, pullNumbers } from "./release-notes.mjs";

const CHANGELOG = `# Changelog

## [Unreleased]

## [1.0.0] - 2026-10-28

First stable release.

### Highlights

- **Session restore:** tabs come back.

### Added

- A folder picker.

### Fixed

- A crash.

### Security

- A hardening.

## [0.4.0] - 2026-10-22

### Added

- Tab reorder.
`;

test("previousStableTag picks the newest stable tag below the release", () => {
  const tags = ["v0.1.0", "v0.10.0", "v0.9.0", "v1.0.0-rc.1", "v1.0.0", "v1.1.0"];
  assert.equal(previousStableTag(tags, "v1.0.0"), "v0.10.0");
  assert.equal(previousStableTag(tags, "v1.0.0-rc.1"), "v0.10.0");
  assert.equal(previousStableTag(tags, "v1.1.0"), "v1.0.0");
  assert.equal(previousStableTag(tags, "v0.1.0"), undefined);
});

test("parseChangelog splits the intro and subsections of one version", () => {
  const parsed = parseChangelog(CHANGELOG, "1.0.0");
  assert.equal(parsed.intro, "First stable release.");
  assert.deepEqual(Object.keys(parsed.sections), ["Highlights", "Added", "Fixed", "Security"]);
  assert.equal(parsed.sections.Fixed, "- A crash.");
  assert.equal(parseChangelog(CHANGELOG, "9.9.9"), null);
  assert.deepEqual(parseChangelog(CHANGELOG, "Unreleased").sections, {});
});

test("pullNumbers reads merge and squash commits and ignores the rest", () => {
  const found = pullNumbers([
    "Merge pull request #12 from a/b\n\nbody",
    "fix(app): thing (#7)",
    "chore: not a pr",
    "Merge pull request #12 from a/b",
    "text (#3) in the middle",
  ]);
  assert.deepEqual(found, [7, 12]);
});

const prs = [
  { number: 5, title: "feat: x @someone", author: "alice" },
  { number: 6, title: "ci: bump y", author: "dependabot[bot]" },
  { number: 7, title: "fix: z", author: "alice" },
  { number: 8, title: "fix: w", author: "bob" },
];

test("compose lists highlights, changes, fixes, pull requests and contributors", () => {
  const body = compose({
    tag: "v1.0.0",
    parsed: parseChangelog(CHANGELOG, "1.0.0"),
    prs,
    previousTag: "v0.4.0",
    repo: "o/r",
  });
  const order = ["## 1.0.0", "### Highlights", "### Changes", "### Fixes", "### Pull requests", "### Contributors"];
  const at = order.map((h) => body.indexOf(h));
  assert.ok(at.every((n, i) => n >= 0 && (i === 0 || n > at[i - 1])), body);
  assert.match(body, /### Changes\n\n- A folder picker\./);
  assert.match(body, /### Fixes\n\n- A crash\.\n- A hardening\./);
  assert.match(body, /\*\*PR #6\*\* ci: bump y\.\n/);
  assert.match(body, /Thanks @alice, @bob\./);
  assert.match(body, /compare\/v0\.4\.0\.\.\.v1\.0\.0/);
});

test("compose never pings a name written in a pull request title", () => {
  const body = compose({ tag: "v1.0.0", parsed: parseChangelog(CHANGELOG, "1.0.0"), prs, repo: "o/r" });
  assert.doesNotMatch(body, /feat: x @someone/);
  assert.match(body, /feat: x @​someone\. Thanks @alice\./);
});

test("compose leaves out empty sections and gives a pre-release a placeholder", () => {
  const body = compose({ tag: "v1.0.0-rc.1", parsed: null, prs: [], repo: "o/r" });
  assert.match(body, /^## 1\.0\.0-rc\.1\n\nPre-release v1\.0\.0-rc\.1/);
  assert.doesNotMatch(body, /### /);
});
