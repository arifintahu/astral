import { test } from "node:test";
import assert from "node:assert/strict";
import { acceptedVersions } from "./version.mjs";

test("a stable tag needs the files at exactly that version", () => {
  assert.deepEqual(acceptedVersions("1.2.0", true), ["1.2.0"]);
});

test("a pre-release tag also accepts the version it is a candidate for", () => {
  // v1.2.0-rc.1 is for trying the release workflow; it should not need a commit per candidate.
  assert.deepEqual(acceptedVersions("1.2.0-rc.1", true), ["1.2.0-rc.1", "1.2.0"]);
});

test("without a tag every file must match the source version exactly", () => {
  // A source of 1.2.0-rc.1 must not let a file at 1.2.0 pass.
  assert.deepEqual(acceptedVersions("1.2.0-rc.1", false), ["1.2.0-rc.1"]);
});
