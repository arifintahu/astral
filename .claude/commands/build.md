Build the full project: frontend first, then the Rust binary.

The Rust binary embeds `web/dist/` via `rust-embed` — always build in this order:

1. `cd web && npm install` (skip if node_modules/ already exists)
2. `cd web && npm run build` → outputs to web/dist/
3. `cargo build` (dev) or `cargo build --release` (optimised)
