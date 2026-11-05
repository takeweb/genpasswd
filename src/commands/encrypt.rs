//! `encrypt`: 旧バージョンで保存した平文のパスワードを一括暗号化する

use super::{load_cipher, open_db};
use crate::crypto;

pub fn run() {
    let cipher = load_cipher();
    let mut db = open_db();
    let rows: Vec<(i64, String)> = db
        .all_passwords()
        .expect("Failed to read history")
        .into_iter()
        .filter(|(_, pwd)| !crypto::is_encrypted(pwd))
        .map(|(id, pwd)| (id, cipher.encrypt(&pwd)))
        .collect();
    if rows.is_empty() {
        println!("All passwords are already encrypted.");
    } else {
        db.replace_passwords(&rows).expect("Failed to encrypt passwords");
        println!("Encrypted {} password(s).", rows.len());
    }
}
