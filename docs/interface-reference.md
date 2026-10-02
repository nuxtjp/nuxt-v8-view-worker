# nuxt-v8-view-worker interface reference

Use the [usage guide](getting-started.md) for the first steps. This reference preserves the current interface details and operational limits. Run command examples from the repository root, after preparing the exact declared dependencies and registered configuration.

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
