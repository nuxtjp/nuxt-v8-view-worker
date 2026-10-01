# nuxt-v8-view-worker

A bounded Rust/V8 process for rendering local scalar input into a limited view fragment. Consumers invoke its versioned stdio contract using explicitly configured executable and renderer digests.

## Components

- `nuxt-v8-view-worker`: one stdin request and one stdout response.
- `nuxt-v8-view-host`: verifies expiring configuration, worker/script SHA-256, and resource limits before launch.
- `src/renderer.js`: fixed pure renderer with HTML escaping.
- `schemas/`: closed request, response, and host-configuration schemas.

The V8 context has no host callbacks or objects. Inputs accept only the declared scalar fields and reject unknown fields, secret values, URLs, paths, and command-like text.

## Execution boundary

Requests are at most 1 MiB. The host validates absolute paths, digests, expiry, and limits, pins Linux file descriptors, clears the child environment, and applies a kill deadline. The worker uses a digest-pinned renderer and returns a bounded response. Heap limits, an internal watchdog, and the host deadline provide separate limits. An OOM exit without a correlated valid response fails closed.

Production deployment additionally supplies OS isolation, such as a dedicated UID, a read-only filesystem, and network namespace/seccomp restrictions. Descriptor execution uses Linux `/proc/self/fd`; other platforms fail closed until an equivalent safe execution path exists.

## Local use

Build the executable, then copy `examples/host-config.example.json` into excluded local configuration and set its actual digest and a short expiry.

```bash
sha256sum target/release/nuxt-v8-view-worker
cargo run --release --bin nuxt-v8-view-host -- /absolute/path/host-config.json \
  < examples/request.json
cargo run --release --bin nuxt-v8-view-worker < examples/request.json
```

Configuration contains environment-specific executable paths and digests and remains outside source control. Reissue it when replacing a worker.

## Verification and build prerequisite

```bash
./scripts/verify-local-foundation.sh
RUSTY_V8_ARCHIVE=/absolute/path/to/reviewed/librusty_v8.a \
  ./scripts/verify-local-foundation.sh
```

The declared `v8` dependency needs its matching `librusty_v8.a` build archive. Supply a separately reviewed archive and verify its SHA-256/provenance. The verification script takes a private temporary snapshot before digest checking and the full-feature gate. It does not implicitly download an archive or overwrite the source input through the V8 build output.

Without an archive, the no-default-feature gate checks schemas, validation, and host termination. It does not establish real rendering or V8 timeout acceptance. Recorded local evidence and its archive digest are in [VALIDATION.md](VALIDATION.md); source publication does not establish production integration.
