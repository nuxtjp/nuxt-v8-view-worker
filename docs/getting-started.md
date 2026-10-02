# Using nuxt-v8-view-worker

Render declared scalar input into a limited view fragment using a verified Rust/V8 worker.

## Before you start

The V8 context exposes no host callbacks. This worker is not a general script runtime or browser.

## First steps

Run from the repository root:

```sh
cargo test --locked
```

## How to assess the result

- Validate a closed input schema.
- Pin worker and renderer digests before a bounded invocation.

A passing source-level check establishes only what that check observes. Keep missing configuration, unavailable services and unverified deployment paths visible.

## Continue reading

[Repository overview](../README.md)
