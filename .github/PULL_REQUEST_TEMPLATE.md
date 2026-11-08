## What and why

<!-- What changed, and why. Link the issue if there is one. -->

## How you checked it

<!-- Commands you ran and what you saw. For interface changes, add a screenshot. -->

- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `cd web && npm run build` and `node scripts/version.mjs check` pass
- [ ] Bug fixes have a test that failed before the fix
- [ ] Changes to data sent over SSE or `/api/*` are mirrored in `web/src/lib/types.ts`
- [ ] No secrets, personal paths or machine names in the diff
- [ ] Docs and `CHANGELOG.md` updated if behaviour changed
