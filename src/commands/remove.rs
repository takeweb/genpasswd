//! `remove`: 履歴を 1 件削除する

use super::{exit_with_error, open_db};

pub fn run(id: i64) {
    let db = open_db();
    match db.delete_entry(id).expect("Failed to delete history") {
        Some(service) => println!("Removed history entry ID {} (service \"{}\").", id, service),
        None => exit_with_error(format!("no history entry with ID {}.", id)),
    }
}
