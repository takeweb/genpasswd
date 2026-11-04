//! `delete`: サービスの全履歴を削除する

use super::open_db;

pub fn run(service: String) {
    let db = open_db();
    let n = db.delete_service(&service).expect("Failed to delete history");
    if n == 0 {
        println!("No history found for service \"{}\".", service);
    } else {
        println!("Deleted {} record(s) for service \"{}\".", n, service);
    }
}
