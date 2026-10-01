# Local V8 validation

- Date: 2026-07-25
- V8 crate: `149.2.0`
- Archive SHA-256:
  `dd548eecb862fb8e746fc0ba4e0d926af94b2ea69299abefbbaac5fa9e3d8799`
- Archive provenance: locally built output of the reviewed Hatter upstream
  snapshot at OpenAI Codex commit
  `89a3b89c4c1d1afaaa93b6669c9e4e03247f8a99`
- Network access during this repository validation: none

The archive was supplied through `RUSTY_V8_ARCHIVE` only while verifying this
repository. It is not a source, package, or runtime dependency on Hatter.

The capacity-gated full tier passed through the shared workspace entrypoint:

```bash
"$WONDERLAND_ROOT/bin/verify-repositories" --rust --tier full
```

Verified behavior includes real rendering, non-yielding JavaScript termination,
host timeout kill, worker digest tamper rejection, and rejection of
secret-, URL-, path-, and command-like values.
