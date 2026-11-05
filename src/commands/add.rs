//! `add`（別名 `register`）: 既存のパスワードを履歴に追加する

use super::{load_cipher, open_db, resolve_username};

pub fn run(service: String, password: String, username: String) {
    let cipher = load_cipher();
    let db = open_db();
    let username = resolve_username(&db, &service, username);
    db.save(&service, &username, &cipher.encrypt(&password)).expect("Failed to save password");
    if username.is_empty() {
        println!("Added to history for service \"{}\".", service);
    } else {
        println!("Added to history for service \"{}\" (user: {}).", service, username);
    }
}
