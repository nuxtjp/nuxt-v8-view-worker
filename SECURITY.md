# Security policy

この worker は信頼境界であり、汎用 JavaScript runtime ではありません。

報告対象は、closed schema の迂回、digest/expiry 検証の迂回、worker の kill 漏れ、
host binding の露出、上限を超える入出力です。入力値やエラーに credential を含めず、
検証用データだけで再現してください。

設定ファイルは secret store ではありません。absolute worker path、binary/script
digest、expiry、resource caps だけを置きます。公開・共有・コミットは禁止です。

V8 archive と release binary は、組織内の provenance 検証、malware scan、SHA-256
記録を経て配布します。archive の取得を Cargo build に暗黙委譲しません。
