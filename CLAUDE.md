# genpasswd_ex

パスワード生成 + サービスごとの履歴管理 CLI（Rust / clap / rusqlite）。使い方は README.md を参照。

## 構成

- `src/main.rs` — エントリーポイント（引数を解析してサブコマンドに振り分けるだけ）
- `src/cli.rs` — CLI 定義（clap derive）
- `src/commands/` — サブコマンドの処理（1 コマンド 1 ファイル、各 `run()`）。引数なしの生成は `generate.rs`。共通ヘルパー（`open_db` / `load_cipher` / `resolve_username` / DB パス解決など）は `commands/mod.rs`
- `src/db.rs` — SQLite アクセス（`password_history` テーブル）
- `src/password.rs` — パスワード生成
- `src/crypto.rs` — パスワードの暗号化/復号（AES-256-GCM）と鍵の取得（macOS キーチェーン / Windows 資格情報マネージャー）
- `src/clipboard.rs` — クリップボードへのコピー（macOS/Linux は外部コマンド `pbcopy` 等、Windows は `clipboard-win`）
- OS 依存部分は `#[cfg(...)]` で分岐（鍵ストア・クリップボード・データディレクトリ）。DB と鍵は端末ごとに別管理

## DB

- テーブル: `password_history (id, service, username, password, created_at)`、`service_tags (service, tag)`（PK は (service, tag)。タグはサービス単位）
- 保存先:
  - release ビルド: `~/.local/share/genpasswd_ex/history.db`（Windows は `%APPDATA%\genpasswd_ex\history.db`）
  - debug ビルド（`cargo run`）: リポジトリ直下の `history_dev.db`
  - 環境変数 `GENPASSWD_EX_DB`（`.env` も可）が設定されていればそれを優先
- DBファイルは開くたびに `600`、データディレクトリは `700` に設定する（unix のみ）
- 本番DB（`history.db`）を更新する場合は、コマンド/SQL を提示してユーザに実行してもらう

## 仕様メモ

- `save` / `add`（旧 `register`、別名として残している）で `-u` 省略時は、同じサービスの履歴にある最新の空でないユーザ名を使う（`resolve_username`）。なければ空。
- 引数なし / `save` で生成したパスワードはクリップボードにコピーする（`--no-copy` で無効化、失敗時は警告のみ）。
- パスワードは `enc:v1:` + base64(nonce 12B + 暗号文) で保存。プレフィックスなしは旧形式の平文として扱い、`encrypt` で一括暗号化（VACUUM で旧データも消去）。
- 鍵はキーチェーン（service `genpasswd_ex`、account は release `db-key` / debug `db-key-dev`）。Windows は資格情報マネージャーの汎用資格情報（ターゲット名 `genpasswd_ex:<account>`）。`GENPASSWD_EX_KEY`（base64 32B）があれば優先。未登録（errSecItemNotFound）の場合のみ新規作成する。
- `update <ID> [-u] [-p]` で既存レコードのユーザ名・パスワードを更新できる（`-u`/`-p` の少なくとも一方が必須、未指定項目は変更しない）。
- `history` は `-s` 省略（全サービス表示）時はパスワードを `********` でマスクし、復号しない（鍵も読まない）。`-s` 指定時は平文表示。
- `copy <ID>` で指定IDのパスワードを復号してクリップボードにコピー（画面には表示しない。コピー失敗時はエラー終了）。
- `remove <ID>` で履歴を1件削除。`delete -s <SERVICE>` はサービスの全履歴を削除。
- `rename -s <OLD> <NEW>` でサービス名を変更。変更先に履歴がある場合はエラー（`--merge` で統合）。タグも引き継ぐ（統合時は和集合）。
- `tag -s <SERVICE> <TAG>...` / `untag` / `tags` でサービスのタグを管理。`tag` は履歴のあるサービスのみ可、タグにカンマ・空白不可、指定時の先頭 `#` は除去。表示は `#a #b` 形式。`delete -s` でタグも削除（`remove` で最後の1件を消してもタグは残るが、履歴のないサービスのタグは表示・絞り込みの対象外）。
- `history -t` でタグ絞り込み。`-t` の複数指定は OR、カンマ区切りは AND（例: `-t common -t nx,stg`）。`-s` とは併用不可、マスク表示。
- `$` を含むパスワードはシェルで展開されるため、引数や SQL はシングルクォートで囲む（`sqlite3` はクォート付きヒアドキュメント `<<'EOF'` を使う）。

## 開発

- ビルド: `cargo build`
- ターゲット別ビルド: `cargo build-mac` / `cargo build-win`（`.cargo/config.toml` のエイリアス。Windows 版のクロスコンパイルには `brew install mingw-w64` と `rustup target add x86_64-pc-windows-gnu` が必要）
- 動作確認は本番DBを汚さないよう `GENPASSWD_EX_DB` にスクラッチの DB パスを指定して行う（鍵も `GENPASSWD_EX_KEY` でテスト用を指定するとキーチェーンを使わない）
- インストール: `cargo install --path . --root ~/`
