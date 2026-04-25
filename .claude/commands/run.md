Start the Astral dev server.

Prerequisite: `web/dist/` must exist. Run `/build` first if it does not.

Default (port 8080, auto-generated credentials):
```bash
cargo run
```

Custom port and credentials:
```bash
cargo run -- --port 3000 --auth admin:secret
```

The server prints the credentials and listening address on startup. Access the dashboard at http://localhost:8080 (or the configured port).
