# genpasswd

パスワード生成 + サービスごとの履歴管理ツール。

パスワードはランダムに生成され、サービス名・ユーザ名と紐づけて SQLite に保存できます。

## ビルド・インストール

### macOS

`~/bin` にインストールする場合:

```
cargo install --path . --root ~/
```

`~/bin` が `$PATH` に含まれていれば、どこからでも `genpasswd` で実行できます。

インストールせずにビルドだけする場合:

```
cargo build --release   # → target/release/genpasswd
cargo build-mac         # → target/aarch64-apple-darwin/release/genpasswd（ターゲット明示）
```

### Windows

#### Windows 上でビルドする

事前に [Rust（rustup）](https://rustup.rs/) と Visual Studio Build Tools（「C++ によるデスクトップ開発」）をインストールしてください（SQLite を C からビルドするため C コンパイラが必要です）。

PowerShell で:

```powershell
# %USERPROFILE%\bin にインストール
cargo install --path . --root $HOME

# ビルドだけする場合 → target\release\genpasswd.exe
cargo build --release
```

`%USERPROFILE%\bin` をユーザー環境変数 `Path` に追加すると、どこからでも `genpasswd` で実行できます。

#### macOS からクロスコンパイルする

初回のみ準備が必要です:

```
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
```

ビルド:

```
cargo build-win   # → target/x86_64-pc-windows-gnu/release/genpasswd.exe
```

生成された `genpasswd.exe` を Windows にコピーし、`Path` の通ったフォルダに置いて使います（単体で動作し、追加の DLL は不要です）。

### ビルド用エイリアス

`cargo build-mac` / `cargo build-win` は `.cargo/config.toml` に定義したエイリアスです。

`--target` を付けない `cargo build --release` は、**ビルドしているマシン自身（ホスト）向け**のバイナリになり、`target/release/` に出力されます（Apple Silicon の Mac なら `aarch64-apple-darwin`、Windows なら通常 `x86_64-pc-windows-msvc`）。`cargo build-mac` との違いはターゲットを明示するかどうかと出力先だけで、Apple Silicon の Mac 上ではどちらも同じ Mac 用バイナリになります。

| コマンド | ターゲット | 出力先 |
|---|---|---|
| `cargo build-mac` | macOS (Apple Silicon) | `target/aarch64-apple-darwin/release/genpasswd` |
| `cargo build-win` | Windows (x86_64) | `target/x86_64-pc-windows-gnu/release/genpasswd.exe` |

## データ保存先

| OS | パス |
|---|---|
| macOS / Linux | `~/.local/share/genpasswd/history.db` |
| Windows | `%APPDATA%\genpasswd\history.db` |

macOS / Linux では、ディレクトリは `700`、DBファイルは `600`（所有者のみ読み書き可）に設定されます。既存のファイルも実行時に同じ権限へ変更されます。

DB と暗号化鍵は端末ごとに別管理です（macOS と Windows で DB を共有する想定はしていません）。

## 旧名 genpasswd_ex からの移行

このツールは以前 `genpasswd_ex` という名前でした（以前の `genpasswd` は [genpasswd_simple](https://github.com/takeweb/genpasswd_simple) に改名）。新しいバージョンは、初回実行時に旧名のデータを自動で引き継ぎます。

| 対象 | 旧 | 新 | 引き継ぎ方 |
|---|---|---|---|
| DB（macOS） | `~/.local/share/genpasswd_ex/history.db` | `~/.local/share/genpasswd/history.db` | 新しい DB がなければコピー |
| DB（Windows） | `%APPDATA%\genpasswd_ex\history.db` | `%APPDATA%\genpasswd\history.db` | 同上 |
| 鍵（macOS） | キーチェーン `genpasswd_ex` / `db-key` | `genpasswd` / `db-key` | 新しい鍵がなければコピー |
| 鍵（Windows） | 資格情報 `genpasswd_ex:db-key` | `genpasswd:db-key` | 同上 |
| 環境変数 | `GENPASSWD_EX_DB` / `GENPASSWD_EX_KEY` | `GENPASSWD_DB` / `GENPASSWD_KEY` | 旧名も引き続き使える（新名を優先） |

- 旧 DB・旧鍵は削除しません。新しいバージョンで問題なく使えることを確認したら、手動で削除してください
- 引き継ぎ後に旧バージョンで追加・変更した内容は、新しい DB には反映されません

旧バージョンのコマンドの削除:

```
# macOS
cargo uninstall genpasswd_ex --root ~/
```

Windows では `genpasswd_ex.exe` を削除し、新しい `genpasswd.exe` を同じフォルダに置いてください。

## 暗号化

パスワードは AES-256-GCM で暗号化して DB に保存されます。暗号化鍵は OS の鍵ストアに保存され、初回の保存時に自動で作成されます。

| OS | 保存先 |
|---|---|
| macOS | キーチェーン（サービス `genpasswd` / アカウント `db-key`） |
| Windows | 資格情報マネージャーの汎用資格情報（ターゲット名 `genpasswd:db-key`） |

- `history` では復号して表示するため、使い方は変わりません
- macOS では、再インストール後などは初回アクセス時にキーチェーンへのアクセス許可ダイアログが表示されます（「常に許可」を選ぶと以後は表示されません）
- **鍵ストアの鍵を削除すると、保存済みのパスワードは復号できなくなります**
- macOS / Windows 以外では、環境変数 `GENPASSWD_KEY` に base64 の 32 バイト鍵を指定してください（`openssl rand -base64 32` で生成可）

旧バージョンで保存した平文のパスワードもそのまま表示できます。暗号化するには次を実行してください（念のため事前に DB をバックアップしてください）。

```
$ genpasswd encrypt
Encrypted 12 password(s).
```

## 使い方

### パスワードを生成する（保存しない）

```
genpasswd [--length <長さ>] [--symbols] [--no-copy]
```

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--length` | `-l` | 16 | パスワードの文字数 |
| `--symbols` | `-k` | off | 記号を含める |
| `--no-copy` | `-n` | off | クリップボードにコピーしない |

```
# 16文字（デフォルト）
$ genpasswd
Generated password: TDeXIVCda7bEWQts
Copied to clipboard.

# 24文字・記号あり
$ genpasswd --length 24 --symbols
Generated password: ]]:dwmstZjQ9scuU1qLRLgLQ
Copied to clipboard.
```

生成されたパスワードは自動でクリップボードにコピーされます（macOS は `pbcopy`、Windows はクリップボード API、Linux は `wl-copy` / `xclip` / `xsel` を使用）。コピーに失敗した場合は警告を表示し、処理は続行します。

---

### パスワードを生成して履歴に保存する

```
genpasswd save --service <SERVICE> [--username <ユーザ名>] [--length <長さ>] [--symbols] [--no-copy]
```

| 引数/オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--service` | `-s` | 必須 | サービス名 |
| `--username` | `-u` | 履歴の最新ユーザ名 | サービスのユーザ名（省略時は同サービスの履歴にある最新のユーザ名。履歴になければ空） |
| `--length` | `-l` | 16 | パスワードの文字数 |
| `--symbols` | `-k` | off | 記号を含める |
| `--no-copy` | `-n` | off | クリップボードにコピーしない |

```
# ユーザ名なし
$ genpasswd save -s github
Generated password: CW5HsZlYxIib2Ut4
Copied to clipboard.
Saved to history for service "github".

# ユーザ名あり・20文字・記号あり
$ genpasswd save -s github --username octocat --length 20 --symbols
Generated password: (*>asM70MQrmdd1mM5bT
Copied to clipboard.
Saved to history for service "github" (user: octocat).
```

生成されたパスワードは標準出力、コピー・保存メッセージは標準エラー出力に書き出されます。

---

### サービスの履歴を確認する

```
genpasswd history [--service <SERVICE> | --tag <TAG>[,<TAG>...]...]
```

`--service`（`-s`）を省略すると、全サービスの履歴をサービスごとに表示します。`--tag`（`-t`）でタグの付いたサービスに絞り込めます（[タグ](#サービスにタグを付ける) を参照）。この場合パスワードは `********` とマスク表示されます（平文で確認するには `-s` でサービスを指定してください）。

```
$ genpasswd history
=== aws (1 record(s)) ==========================================================
  ID  Username              Password              Created At
--------------------------------------------------------------------------------
   3  -                     ********              2026-07-07T08:10:12.512331+09:00

=== github (1 record(s)) =======================================================
  ID  Username              Password              Created At
--------------------------------------------------------------------------------
   7  octocat               ********              2026-07-07T08:07:54.151484+09:00
```

```
$ genpasswd history -s github
=== github (3 record(s)) =======================================================
  ID  Username              Password              Created At
--------------------------------------------------------------------------------
   7  octocat               YWXWA3xS402Wl5JKMmsQ  2026-07-07T08:07:54.151484+09:00
   5  octocat               9Jn2UoOc5EMMofvB      2026-07-07T08:06:01.893545+09:00
   1  -                     (*>asM70MQrmdd1mM5bT  2026-07-07T08:01:39.249806+09:00
```

サービスの見出しは `=` の罫線で表示されます（端末への出力時は太字・色付き。環境変数 `NO_COLOR` を設定すると色なし）。タグが付いている場合は見出しに `#nx #stg` のように表示されます。新しいパスワードが先頭に表示されます。ユーザ名未設定の場合は `-` と表示されます。

---

### 履歴のパスワードをクリップボードにコピーする

```
genpasswd copy <ID>
```

```
$ genpasswd copy 7
Copied password of ID 7 (service "github", user: octocat) to clipboard.
```

ID は `history` で確認してください（`-s` を省略したマスク表示の ID でも使えます）。パスワードは画面に表示されず、クリップボードにのみコピーされます。コピーに失敗した場合はエラーになります。

---

### 登録済みサービス一覧を表示する

```
genpasswd list
```

```
$ genpasswd list
Service  Count  Tags
--------------------------
aws          2  #private
github       4
```

タグが付いているサービスは `Tags` 列に表示されます。

---

### サービスにタグを付ける

サービスにタグを付けておくと、`history -t` で表示を絞り込めます。顧客・環境（共通 / STG / 本番）などでグルーピングする用途を想定しています。タグはサービス単位で付くため、同じサービスのユーザ・履歴をすべて含みます。

```
genpasswd tag   --service <SERVICE> <TAG>...   # タグを付ける
genpasswd untag --service <SERVICE> <TAG>...   # タグを外す
genpasswd tags                                 # タグごとのサービス一覧
```

```
$ genpasswd tag -s 'BIG-IP Edge Client' nx common
Added 2 tag(s) to service "BIG-IP Edge Client": #common #nx
$ genpasswd tag -s 'バッチサーバ(ステージング用)' nx stg
Added 2 tag(s) to service "バッチサーバ(ステージング用)": #nx #stg
$ genpasswd tag -s 'バッチサーバ(本番用)' nx prod
Added 2 tag(s) to service "バッチサーバ(本番用)": #nx #prod
```

`history -t` の指定方法:

| 指定 | 意味 |
|---|---|
| `-t stg` | `stg` タグの付いたサービス |
| `-t common -t stg` | `common` **または** `stg`（`-t` を複数指定すると OR） |
| `-t nx,stg` | `nx` **かつ** `stg`（カンマ区切りは AND） |
| `-t nx,common -t nx,stg` | (`nx` かつ `common`) または (`nx` かつ `stg`) |

```
# 共通 + STG の設定だけを表示
$ genpasswd history -t common -t stg
=== BIG-IP Edge Client (1 record(s)) #common #nx ===============================
...
=== バッチサーバ(ステージング用) (2 record(s)) #nx #stg ========================
...
```

- `-t` は `-s` と同時には指定できません。`-s` を省略したときと同じく、パスワードはマスク表示されます
- タグは `#nx #stg` の形式で表示されます。指定するときは `#` なしで書きます（`#` はシェルでコメント扱いになるため。`'#stg'` のようにクォートすれば `#` 付きでも可）
- タグ名にカンマ・空白は使えません。大文字・小文字は区別されます
- タグを付けられるのは履歴のあるサービスだけです（サービス名の打ち間違い防止）
- `rename` するとタグも新しいサービス名に引き継がれます（`--merge` の場合は両方のタグを合わせたものになります）。`delete` でサービスの履歴を削除するとタグも削除されます

---

### 既存のパスワードを履歴に追加する

```
genpasswd add --service <SERVICE> <PASSWORD> [--username <ユーザ名>]
```

| 引数/オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--service` | `-s` | 必須 | サービス名 |
| `PASSWORD` | — | 必須 | 追加するパスワード |
| `--username` | `-u` | 履歴の最新ユーザ名 | サービスのユーザ名（省略時は同サービスの履歴にある最新のユーザ名。履歴になければ空） |

```
$ genpasswd add -s github myP@ssw0rd --username octocat
Added to history for service "github" (user: octocat).
```

旧コマンド名 `register` も別名として引き続き使えます。

---

### 履歴のユーザ名・パスワードを更新する

```
genpasswd update <ID> [--username <ユーザ名>] [--password <パスワード>]
```

| 引数/オプション | 短縮 | 説明 |
|---|---|---|
| `ID` | — | 更新する履歴のID（`history` で確認） |
| `--username` | `-u` | 新しいユーザ名 |
| `--password` | `-p` | 新しいパスワード |

`-u` と `-p` の少なくとも一方が必要です。指定しなかった項目は変更されません。

```
$ genpasswd update 16 -u octocat
Updated history entry ID 16.

$ genpasswd update 15 -p 'C$Y3xvqiRsHF2H.'
Updated history entry ID 15.
```

`$` などを含むパスワードはシェルに展開されないようシングルクォートで囲んでください。

---

### サービス名を変更する

```
genpasswd rename --service <SERVICE> <NEW_SERVICE> [--merge]
```

| 引数/オプション | 短縮 | 説明 |
|---|---|---|
| `--service` | `-s` | 現在のサービス名（必須） |
| `NEW_SERVICE` | — | 新しいサービス名（必須） |
| `--merge` | なし | 変更先のサービスが既にある場合に履歴を統合する |

```
$ genpasswd rename -s google Google
Renamed service "google" to "Google" (1 record(s)).
```

変更先のサービス名に履歴が既にある場合は、誤って統合しないようエラーになります。統合する場合は `--merge` を指定してください。

---

### 履歴を1件削除する

```
genpasswd remove <ID>
```

```
$ genpasswd remove 5
Removed history entry ID 5 (service "github").
```

ID は `history` で確認してください。

---

### サービスの履歴を削除する

```
genpasswd delete --service <SERVICE>
```

```
$ genpasswd delete -s aws
Deleted 2 record(s) for service "aws".
```

指定したサービスのすべての履歴が削除されます。

## ライセンス

MIT License

Copyright (c) 2026 Taketomo Oishi

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
