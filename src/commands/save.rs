//! `save`: パスワードを生成して履歴に保存する

use super::{generate_and_show, load_cipher, open_db, resolve_username};

pub fn run(service: String, username: String, length: usize, symbols: bool, no_copy: bool) {
    let pwd = generate_and_show(length, symbols, no_copy);

    let cipher = load_cipher();
    let db = open_db();
    let username = resolve_username(&db, &service, username);
    db.save(&service, &username, &cipher.encrypt(&pwd)).expect("Failed to save password");
    if username.is_empty() {
        eprintln!("Saved to history for service \"{}\".", service);
    } else {
        eprintln!("Saved to history for service \"{}\" (user: {}).", service, username);
    }
}
