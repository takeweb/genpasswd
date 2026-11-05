//! サブコマンドの処理（1 コマンド 1 ファイル）と共通ヘルパー

pub mod add;
pub mod copy;
pub mod delete;
pub mod encrypt;
pub mod generate;
pub mod history;
pub mod list;
pub mod remove;
pub mod rename;
pub mod save;
pub mod tag;
pub mod tags;
pub mod untag;
pub mod update;

use crate::{clipboard, crypto, db, password};
use std::path::PathBuf;

/// エラーメッセージを表示して終了する
fn exit_with_error(msg: impl std::fmt::Display) -> ! {
    eprintln!("Error: {}", msg);
    std::process::exit(1);
}

/// パスワードを生成して表示し、必要ならクリップボードにコピーする
fn generate_and_show(length: usize, symbols: bool, no_copy: bool) -> String {
    if length < 4 {
        exit_with_error("password length must be at least 4.");
    }
    let pwd = password::generate_password(length, symbols);
    println!("Generated password: {}", pwd);
    if !no_copy {
        copy_to_clipboard(&pwd);
    }
    pwd
}

/// ユーザ名が省略された場合、同じサービスの履歴にある最新のユーザ名を使う
fn resolve_username(db: &db::Db, service: &str, username: String) -> String {
    if !username.is_empty() {
        return username;
    }
    db.latest_username(service)
        .expect("Failed to read history")
        .unwrap_or_default()
}

/// 生成したパスワードをクリップボードにコピーする（失敗しても処理は続行）
fn copy_to_clipboard(pwd: &str) {
    match clipboard::copy(pwd) {
        Ok(()) => eprintln!("Copied to clipboard."),
        Err(e) => eprintln!("Warning: failed to copy to clipboard ({}).", e),
    }
}

/// 暗号化鍵を読み込む（失敗したら終了）
fn load_cipher() -> crypto::Cipher {
    crypto::Cipher::load().unwrap_or_else(|e| exit_with_error(e))
}

/// タグ指定から先頭の `#` を 1 つ取り除き、前後の空白を除去する（`-t '#stg'` も `stg` として扱う）
fn strip_tag(tag: &str) -> &str {
    let tag = tag.trim();
    tag.strip_prefix('#').unwrap_or(tag).trim()
}

/// コマンドラインで指定されたタグを検証して整える（空・カンマ入り・空白入りはエラー、重複は除去）
fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for tag in tags {
        let tag = strip_tag(&tag).to_string();
        if tag.is_empty() {
            exit_with_error("tag must not be empty.");
        }
        if tag.contains(',') {
            exit_with_error(format!("tag must not contain a comma: \"{}\".", tag));
        }
        // `#a #b` 形式で表示するため、空白を含むタグは区切りが分からなくなるので不可
        if tag.chars().any(char::is_whitespace) {
            exit_with_error(format!("tag must not contain whitespace: \"{}\".", tag));
        }
        if !result.contains(&tag) {
            result.push(tag);
        }
    }
    result
}

/// タグを `#a #b` 形式にする（タグなしは空文字）
fn format_tags(tags: &[String]) -> String {
    tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")
}

/// DB を開く
fn open_db() -> db::Db {
    db::Db::open(&db_path()).expect("Failed to open database")
}

/// データディレクトリ名
const APP_DIR: &str = "genpasswd";
/// 旧ツール名（genpasswd_ex）時代のデータディレクトリ名。新しい DB がなければここから引き継ぐ
const LEGACY_APP_DIR: &str = "genpasswd_ex";

/// データディレクトリ（Windows: `%APPDATA%\<name>` / それ以外: `~/.local/share/<name>`）
#[cfg(windows)]
fn data_dir(name: &str) -> PathBuf {
    dirs::data_dir().expect("Cannot determine data directory").join(name)
}

#[cfg(not(windows))]
fn data_dir(name: &str) -> PathBuf {
    let home = dirs::home_dir().expect("Cannot determine home directory");
    home.join(".local/share").join(name)
}

/// DB のパス（環境変数 `GENPASSWD_DB`、旧名 `GENPASSWD_EX_DB` があれば優先）
fn db_path() -> PathBuf {
    if let Some(path) = ["GENPASSWD_DB", "GENPASSWD_EX_DB"].into_iter().find_map(|name| std::env::var(name).ok()) {
        return PathBuf::from(path);
    }

    let dir = data_dir(APP_DIR);
    std::fs::create_dir_all(&dir).expect("Failed to create data directory");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .expect("Failed to set permissions of data directory");
    }
    if cfg!(debug_assertions) {
        return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("history_dev.db");
    }

    let path = dir.join("history.db");
    migrate_legacy_db(&path);
    path
}

/// 新しい DB がまだなく、旧ディレクトリに DB があればコピーして引き継ぐ（旧 DB は削除しない）
fn migrate_legacy_db(path: &std::path::Path) {
    if path.exists() {
        return;
    }
    let legacy = data_dir(LEGACY_APP_DIR).join("history.db");
    if !legacy.exists() {
        return;
    }
    std::fs::copy(&legacy, path).expect("Failed to copy database from the legacy data directory");
    eprintln!("Copied database from {} to {}.", legacy.display(), path.display());
}
