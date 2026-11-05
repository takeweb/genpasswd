//! `rename`: サービス名を変更する（変更先に履歴があれば `--merge` が必要）

use super::{exit_with_error, open_db};

pub fn run(service: String, new_service: String, merge: bool) {
    if service == new_service {
        exit_with_error("new service name is the same as the current one.");
    }
    let db = open_db();
    let existing = db.count_service(&new_service).expect("Failed to read history");
    if existing > 0 && !merge {
        exit_with_error(format!(
            "service \"{}\" already has {} record(s). Use --merge to merge the histories.",
            new_service, existing
        ));
    }
    let n = db.rename_service(&service, &new_service).expect("Failed to rename service");
    if n == 0 {
        exit_with_error(format!("no history found for service \"{}\".", service));
    }
    println!("Renamed service \"{}\" to \"{}\" ({} record(s)).", service, new_service, n);
}
