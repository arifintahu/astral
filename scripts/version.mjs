// One version for the whole repo: Cargo.toml is the source; Cargo.lock, web/package.json and
// web/package-lock.json must match it.
//   node scripts/version.mjs check [vX.Y.Z]   fail if any file disagrees (or with the given tag)
//   node scripts/version.mjs set X.Y.Z         write the version everywhere
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const CARGO = "Cargo.toml";
const CARGO_LOCK = "Cargo.lock";
const PACKAGE = "web/package.json";
const PACKAGE_LOCK = "web/package-lock.json";
const SEMVER = /^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/;
const JSON_VERSION = /^(\s*"version":\s*)"[^"]+"/m;

const cargoVersion = () => readFileSync(CARGO, "utf8").match(/^version = "([^"]+)"/m)?.[1];
const lockVersion = () =>
  readFileSync(CARGO_LOCK, "utf8").replace(/\r\n/g, "\n").match(/\[\[package\]\]\nname = "astral"\nversion = "([^"]+)"/)?.[1];
const pkgVersion = () => JSON.parse(readFileSync(PACKAGE, "utf8")).version;
const pkgLock = () => JSON.parse(readFileSync(PACKAGE_LOCK, "utf8"));

/**
 * The versions the files may have. A tag needs them to match exactly, except that a pre-release
 * tag (v1.2.0-rc.1) is for trying the release workflow, so files at the version it is a candidate
 * for (1.2.0) pass too and a candidate does not need a version commit of its own. Without a tag
 * every file must equal the source version.
 */
export function acceptedVersions(want, isTag) {
  return isTag && want.includes("-") ? [want, want.split("-")[0]] : [want];
}

function check(tag) {
  const lock = pkgLock();
  const found = {
    [CARGO]: cargoVersion(),
    [CARGO_LOCK]: lockVersion(),
    [PACKAGE]: pkgVersion(),
    [`${PACKAGE_LOCK} (version)`]: lock.version,
    [`${PACKAGE_LOCK} (root package)`]: lock.packages?.[""]?.version,
  };
  const want = tag ? tag.replace(/^v/, "") : found[CARGO];
  const accepted = acceptedVersions(want, Boolean(tag));
  const bad = Object.entries(found).filter(([, v]) => !accepted.includes(v));
  if (bad.length) {
    console.error(`Expected version ${want}, found:`);
    for (const [file, v] of bad) console.error(`  ${file}: ${v ?? "(none)"}`);
    process.exit(1);
  }
  console.log(`All versions are ${want}`);
}

function set(version) {
  if (!SEMVER.test(version ?? "")) { console.error(`Not a version: ${version}`); process.exit(1); }
  const quoted = `"${version}"`;

  writeFileSync(CARGO, readFileSync(CARGO, "utf8").replace(/^version = "[^"]+"/m, `version = ${quoted}`));
  writeFileSync(
    CARGO_LOCK,
    readFileSync(CARGO_LOCK, "utf8").replace(/(\[\[package\]\]\r?\nname = "astral"\r?\nversion = )"[^"]+"/, `$1${quoted}`),
  );
  writeFileSync(PACKAGE, readFileSync(PACKAGE, "utf8").replace(JSON_VERSION, `$1${quoted}`));

  // package-lock.json states the version twice: at the top and under packages[""].
  let lock = readFileSync(PACKAGE_LOCK, "utf8");
  lock = lock.replace(JSON_VERSION, `$1${quoted}`);
  lock = lock.replace(/("packages":\s*\{\s*"":\s*\{[^}]*?"version":\s*)"[^"]+"/, `$1${quoted}`);
  writeFileSync(PACKAGE_LOCK, lock);

  check();
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [cmd, arg] = process.argv.slice(2);
  if (cmd === "check") check(arg);
  else if (cmd === "set") set(arg);
  else { console.error("usage: version.mjs check [vX.Y.Z] | set X.Y.Z"); process.exit(1); }
}
