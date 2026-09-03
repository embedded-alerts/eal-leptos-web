# Environment files

Secrets for `embedded-alerts/eal-leptos-web` are committed only as SOPS+age
ciphertext under `env/enc/`, following the `ORESoftware/ores-sops` contract.
Plaintext is disposable, local-only state under `env/dec/`.

```text
env/enc/dev.env.enc     committed ciphertext; development source of truth
env/enc/prod.env.enc    committed ciphertext; production recipient set
env/dec/dev.env         ignored plaintext; mode 0600; disposable
env/dec/prod.env        ignored plaintext; mode 0600; disposable
.env                    managed relative symlink into env/dec/
```

`env/dec` is a build artifact. Delete it at any time and regenerate it with
`just env-decrypt`. The root `.env` may only be a symlink created by
`just env-use`; recipes refuse to replace an unmanaged file.

## First run

```sh
just env-keygen
just env-whoami
```

Send only the printed public recipient to a maintainer. After the recipient is
added to `.sops.yaml` and ciphertext is rekeyed:

```sh
just env-decrypt
just env-use dev
just env-check
```

CI intentionally has no age identity. `just env-check` is key-independent: it
verifies repository history, ignore rules, encrypted-file structure, and the
minimum recipient count without decrypting values.

## Daily commands

| Command | Purpose |
| --- | --- |
| `just env-list` | list environments and variable names, never values |
| `just env-decrypt [name…]` | materialize ignored plaintext with mode 0600 |
| `just env-use <name>` | point `.env` at one managed plaintext file |
| `just env-unuse` | remove the managed `.env` symlink |
| `just env-edit <name>` | edit and re-encrypt through the reviewed recipe |
| `just env-encrypt [name…]` | fold local edits back into ciphertext |
| `just env-status` | show which variable names differ |
| `just env-run <name> <cmd…>` | inject values directly into one child process |
| `just env-new <name>` | create a new encrypted environment |
| `just env-rekey` | apply the current recipient policy |
| `just env-check` | fail-closed repository audit |
| `just env-doctor` | report required tools and local key state |
| `just env-clean` | remove all ignored plaintext |

Use the recipes rather than invoking SOPS directly. Direct re-encryption changes
every initialization vector, destroys useful diffs, and increases merge risk.

## Value constraints

Variable names remain plaintext for reviewability; values and comments are
encrypted. Never place a secret in a variable name. Dotenv values cannot span
physical lines. Store PEM-like material as one value with escaped newlines; do
not paste credential headers or realistic credential examples into repository
documentation because the repository scanner intentionally rejects
credential-shaped text.

## Containers

Decrypt at runtime, never during `docker build`. Removing a secret in a later
layer does not remove it from image history, and build arguments are also
observable. For local development:

```sh
just env-docker-run dev ghcr.io/embedded-alerts/eal-leptos-web:dev
```

For production, prefer platform-managed secrets:

```sh
just env-k8s-secret prod | kubectl apply -f -
```

Host-side `--env-file` injection keeps the application as PID 1, but those
values are visible to `docker inspect`; it is not the production authority.

## Rules

- Never commit anything under `env/dec/` or an unmanaged `.env`.
- Never commit a private age identity.
- Removing an encryption recipient does not rotate an exposed application
  credential; rotate the application credential separately.
- Keep `.just/env.just` and `.just/dotenv.py` byte-identical to the reviewed
  fleet module; do not fork local semantics.
- Any legitimate non-secret `*.env` file requires a narrow explicit allow rule;
  deny by default.

Compatibility aliases remain available: `env-audit`→`env-check`,
`env-lock`→`env-clean`, `env-key`→`env-whoami`, `env-diff`→`env-status`, and
`env-refresh`→`env-decrypt`.
