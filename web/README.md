# Astral web UI

The Svelte 5 frontend for [Astral](../README.md). `npm run build` writes `web/dist`, which the Rust binary embeds with `rust-embed`, so build it before `cargo build`.

```sh
npm ci
npm run build   # production build; also the type-check step
npm run dev     # Vite with hot reload (no /api proxy, so it cannot sign in)
```

See [CONTRIBUTING.md](../CONTRIBUTING.md) for code style and [CLAUDE.md](../CLAUDE.md) for the component map.
