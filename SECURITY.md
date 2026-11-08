# Security policy

Astral serves live system metrics and, if you turn it on, the list of running processes, so security reports are taken seriously.

## Report a vulnerability

Please do **not** open a public issue. Use GitHub's private reporting instead: open the repository's **Security** tab and choose **Report a vulnerability**. Include what you found, how to reproduce it, and the version (`astral --version`).

You can expect a first answer within a few days. Astral is maintained by one person, so fixes are made on a best-effort basis.

## Supported versions

Only the latest release gets security fixes.

## Scope

In scope: sign-in and session handling (token signing, cookies, revocation, rate limiting), the `/api/*` endpoints and their authorization, the security headers, webhook handling (including SSRF), the embedded static file server, the Docker setup, and the release workflow.

Not treated as vulnerabilities:

- Anyone who has valid credentials can read all metrics and change settings. That is what the credentials are for.
- Running Astral over plain HTTP exposes the login and the session cookie. Astral does not terminate TLS itself; put it behind a reverse proxy (see the README).
- Running the Docker image with `--pid host` and the `/proc` and `/sys` mounts shows host process names to signed-in users. The README explains why those are needed.
- Credentials set with `--auth` are visible in the process list. Use `ASTRAL_AUTH` instead.

## Network use

Astral has no account system, no telemetry and no update check. The only requests the server makes on its own go to the alert webhook URLs you configure, and only over HTTPS.

The dashboard loads the Geist fonts from Google Fonts in your browser. If that is a problem for you, host the fonts yourself and change the `<link>` in `web/index.html`.

## Notes for operators

- Keep `.env` and your credentials out of the repository and out of issues. Never paste the contents of `.env` or a webhook URL into an issue.
- `astral.db` holds your metrics history. Treat it like any other data file on the server.
- Bind the port to localhost (the default in `docker-compose.yml`) and expose it through a TLS-terminating reverse proxy.
