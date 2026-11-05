//! `list`: サービス一覧と件数を表示する

use super::open_db;
use unicode_width::UnicodeWidthStr;

pub fn run() {
    let db = open_db();
    let services = db.list_services().expect("Failed to list services");
    if services.is_empty() {
        println!("No services found.");
        return;
    }
    let col = services
        .iter()
        .map(|(svc, _)| svc.width())
        .max()
        .unwrap_or(0)
        .max("Service".len());
    println!("{}  Count", pad("Service", col));
    println!("{}", "-".repeat(col + 8));
    for (svc, cnt) in &services {
        println!("{}  {:>5}", pad(svc, col), cnt);
    }
}

/// 表示幅（全角は 2）を考慮して右側を空白で埋める
fn pad(s: &str, width: usize) -> String {
    let spaces = width.saturating_sub(s.width());
    format!("{}{}", s, " ".repeat(spaces))
}
