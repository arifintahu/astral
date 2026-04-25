Run static analysis on both Rust and frontend code.

1. `cargo check` — verify Rust compiles
2. `cargo clippy -- -D warnings` — lint with warnings-as-errors
3. `cd web && npm run build` — frontend type check (Vite catches TypeScript/Svelte errors)

Run in sequence; report errors from each step before continuing.
