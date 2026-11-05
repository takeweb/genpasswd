//! `copy`: 指定 ID のパスワードを復号してクリップボードにコピーする

use super::{exit_with_error, load_cipher, open_db};
use crate::clipboard;

pub fn run(id: i64) {
    let db = open_db();
    let Some((service, username, stored)) = db.get_entry(id).expect("Failed to read history") else {
        exit_with_error(format!("no history entry with ID {}.", id));
    };
    let pwd = load_cipher().decrypt(&stored).unwrap_or_else(|e| exit_with_error(e));
    if let Err(e) = clipboard::copy(&pwd) {
        exit_with_error(format!("failed to copy to clipboard ({}).", e));
    }
    if username.is_empty() {
        println!("Copied password of ID {} (service \"{}\") to clipboard.", id, service);
    } else {
        println!("Copied password of ID {} (service \"{}\", user: {}) to clipboard.", id, service, username);
    }
}
