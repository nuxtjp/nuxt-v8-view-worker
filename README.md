# nuxt-v8-view-worker

許可された単純な入力から、制限されたV8環境で表示断片を生成できます。

## 利用前の確認

実装済みの範囲、必要な依存関係、検証コマンドを以下の英語説明に併記しています。操作・配備・公開は、それぞれの権限と設定を確認してから実施してください。

## 使い方

リポジトリ内のサンプル・スキーマ・実装を確認し、用途に必要な入力を明示して利用します。下記のGetting startedに、現行設定に対応する検証コマンドを示しています。

検証結果は実行した範囲だけを示します。未実装の機能、未設定の接続、配備環境の確認を合格扱いにしないでください。

## English

Render declared scalar input into a limited view fragment using a verified Rust/V8 worker.

## What you can do

- Validate a closed input schema.
- Pin worker and renderer digests before a bounded invocation.

## Current scope

The V8 context exposes no host callbacks. This worker is not a general script runtime or browser.

Package distribution is not activated by this documentation. Use the checked-in source and the declared dependency versions; published availability must be verified separately.

## Getting started

Install Rust 1.97 or newer and make the declared dependencies available. Use the configured private registry when a dependency is not distributed publicly. Run from this repository:

```sh
cargo test --locked
```

## Examples and interface details

## Components

- `nuxt-v8-view-worker`: one stdin request and one stdout response.
- `nuxt-v8-view-host`: verifies expiring configuration, worker/script SHA-256, and resource limits before launch.
- `src/renderer.js`: fixed pure renderer with HTML escaping.
- `schemas/`: closed request, response, and host-configuration schemas.

The V8 context has no host callbacks or objects. Inputs accept only the declared scalar fields and reject unknown fields, secret values, URLs, paths, and command-like text.

## Documentation and source

[Interface reference](docs/interface-reference.md)

[Usage guide](docs/getting-started.md)

[Examples](examples) · [Schemas](schemas) · [Implementation and public interfaces](src) · [Verification cases](tests) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)
