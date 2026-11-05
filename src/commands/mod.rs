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

/// DB を開く
fn open_db() -> db::Db {
    db::Db::open(&db_path()).expect("Failed to open database")
}

/// データディレクトリ（Windows: `%APPDATA%\genpasswd_ex` / それ以外: `~/.local/share/genpasswd_ex`）
#[cfg(windows)]
fn data_dir() -> PathBuf {
    dirs::data_dir().expect("Cannot determine data directory").join("genpasswd_ex")
}

#[cfg(not(windows))]
fn data_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Cannot determine home directory");
    home.join(".local/share/genpasswd_ex")
}

fn db_path() -> PathBuf {
    if let Ok(path) = std::env::var("GENPASSWD_EX_DB") {
        return PathBuf::from(path);
    }

    let dir = data_dir();
    std::fs::create_dir_all(&dir).expect("Failed to create data directory");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .expect("Failed to set permissions of data directory");
    }
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("history_dev.db")
    } else {
        dir.join("history.db")
    }
}
