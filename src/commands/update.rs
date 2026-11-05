//! `update`: 既存レコードのユーザ名・パスワードを更新する

use super::{exit_with_error, load_cipher, open_db};

pub fn run(id: i64, username: Option<String>, password: Option<String>) {
    let password = password.map(|p| load_cipher().encrypt(&p));
    let db = open_db();
    let n = db
        .update_entry(id, username.as_deref(), password.as_deref())
        .expect("Failed to update history");
    if n == 0 {
        exit_with_error(format!("no history entry with ID {}.", id));
    }
    println!("Updated history entry ID {}.", id);
}
