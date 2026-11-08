# Contributing to Astral

Astral is a small, single-binary monitoring dashboard; fixes and focused features are welcome. For anything big,
open an issue first so we agree on the shape before you write it.

## Setup

Prerequisites: [Rust](https://rustup.rs/) (stable, edition 2024) and [Node.js](https://nodejs.org/) 20+ with npm.

```sh
git clone https://github.com/arifintahu/astral.git
cd astral
cd web && npm ci && npm run build && cd ..   # the frontend is embedded into the binary
cargo run -- --auth admin:secret --port 3000
```

**Build order matters.** The Rust binary embeds `web/dist/` with `rust-embed`, so build the frontend before
`cargo build`. A missing or stale `web/dist/` compiles without error and serves the wrong files.

For frontend work, `cd web && npm run dev` starts Vite with hot reload, but it has no `/api` proxy, so it cannot
sign in. Rebuild the frontend and restart the binary to try a change end to end.

## Before you open a PR

CI runs on every pull request, but run these first. All of them must pass; say in the PR that you ran them.

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cd web && npm ci && npm run build   # the frontend type-check; there is no separate lint step
node scripts/version.mjs check
node --test scripts/*.test.mjs
```

## Code style

- **Tests first for logic and bug fixes.** Write the failing test, then the code. Skip tests for pure layout and
  glue. Rust tests live in a `*_test.rs` file next to the source (`#[cfg(test)] #[path = "x_test.rs"] mod tests;`).
- **Rust:** edition 2024 idioms. `anyhow::Result` for fallible cross-module functions, `?` instead of `unwrap()` or
  `expect()` (those are fine in tests and in `main()` startup). No `#[allow(...)]` without a comment saying why.
- **Svelte 5 runes only** (`$state`, `$derived`, `$effect`). No Svelte 4 reactive syntax.
- **Tailwind utility classes only,** no `<style>` blocks. Colours and shared classes come from the tokens in
  `web/src/app.css`.
- **No `any`.** New data shapes go in `web/src/lib/types.ts`.
- **TypeScript mirrors Rust.** A field added, removed or renamed in a struct sent over SSE or the history API must be
  mirrored in `web/src/lib/types.ts` in the same change. The frontend does not validate what it receives.
- **The SSE event shape is a stable contract.** Add fields freely; do not rename or remove existing ones.
- **No hard-coded port or credentials.** They come from `--port` and `--auth` / `ASTRAL_AUTH`.
- **No new runtime dependencies without a reason.** Astral ships as one self-contained binary.
- **Comments say why**, not what. No TODO or FIXME in committed code; open an issue instead.
- Keep changes surgical: touch only what the change needs, and match the surrounding style.

See [`CLAUDE.md`](CLAUDE.md) for the architecture and file map.

## Commits and pull requests

- [Conventional Commits](https://www.conventionalcommits.org/): `type(scope): imperative subject`, where type is
  `feat`, `fix`, `chore`, `refactor`, `docs`, `test`, `perf`, `build` or `ci`. Explain *why* in the body when it is
  not obvious.
- Branch off `main`; do not commit to `main` directly.
- A PR title in the same style, a short what and why, and how you verified it (commands run, or what you tried in
  the browser). For interface changes, add a screenshot. The PR template has the checklist.
- Do not commit secrets, personal paths or machine names. Update `CHANGELOG.md` under `[Unreleased]` when behavior
  changes.
- Report security problems privately, see [SECURITY.md](SECURITY.md).

## Versioning and releases

One version for the whole repo. `Cargo.toml` is the source; `Cargo.lock`, `web/package.json` and
`web/package-lock.json` must match it, and CI fails if they drift.

```sh
node scripts/version.mjs set 1.2.0     # writes all four files
node scripts/version.mjs check         # verifies they agree
```

To release: bump the version in a PR and move the `[Unreleased]` entries of `CHANGELOG.md` under a
`## [X.Y.Z] - date` heading, with a `### Highlights` list on top, merge it, then push the tag. The GitHub release
body is built from that section (Highlights; Changes from Added, Changed and Removed; Fixes from Fixed and
Security) plus the pull requests and contributors since the previous release. The Release workflow fails for a
stable tag without Highlights. Preview the notes before tagging with:

```sh
GITHUB_TOKEN=$(gh auth token) GITHUB_REPOSITORY=arifintahu/astral node scripts/release-notes.mjs vX.Y.Z main
```

```sh
git checkout main && git pull
git tag v1.2.0 && git push origin v1.2.0
```

The Release workflow refuses a tag that disagrees with the files, builds binaries for Linux (amd64), Windows (amd64)
and macOS (Apple Silicon), and publishes them with those notes. A tag with a suffix (`v1.2.0-rc.1`) is published as
a pre-release, and only needs the files to be at the version it is a candidate for (1.2.0), so trying the workflow
takes no version commit. A stable tag must match the files exactly. Delete a candidate's release and tag when done.

## License

By contributing you agree that your contributions are licensed under the [MIT License](LICENSE).
