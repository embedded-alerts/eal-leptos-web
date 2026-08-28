# eal-leptos-web

Server-rendered Leptos user dashboard for Embedded Alerts.

The application is an API client, not a second index, database, crawler, realtime bus, or delivery engine. It shows registered source boundaries, immutable page revisions, candidate matches, and a natural-language interest preview. Preview searches deliberately omit an alert-rule revision, so they cannot create candidates or send notifications.

## Development

```bash
cp .env.example .env
cargo run
```

Required configuration:

- `APP_ENV=development|test`; production intentionally fails startup.
- `EAL_API_BASE_URL` points to `eal-api` and may not contain URL credentials.
- `EAL_TENANT_ID` is the temporary server-side tenant selector until Shared Auth claims are certified.
- `HOST` and `PORT` configure the listener.

The API client blocks redirects, uses connection/request timeouts, caps decoded responses at 4 MiB, and never accepts tenant identity from browser form/query input.

## Production gates

1. Shared Auth replaces the development tenant header.
2. Alert rules, source/page revisions, embeddings, and candidates use tenant-scoped PostgreSQL/pgvector repositories.
3. Authenticated tenant-filtered events replace process-local WebSockets.
4. DEN-3460 provides a durable outbox, cooldown/grouping, provider idempotency, receipts, retries, and dead letters.
5. Origin/CSP and cross-tenant/restart canaries pass in `embedded-alerts-test`.

## Validation

```bash
python3 scripts/verify_repo.py
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Linear: DEN-3461; related DEN-3459, DEN-3460, DEN-3462.

## Environment secrets

Secrets live in this repo **encrypted** with [sops](https://github.com/getsops/sops) + [age](https://github.com/FiloSottile/age):
`env/enc/<dev|prod>.env.enc` is committed; `just env-use <name>` decrypts it to
`env/dec/<name>.env` (gitignored, mode 0600) and symlinks `./.env` to it. The
Nix dev shell provides the tooling, `just env-audit` runs keyless in CI, and
containers decrypt at `docker run` — never at build. See [`env/README.md`](env/README.md).
