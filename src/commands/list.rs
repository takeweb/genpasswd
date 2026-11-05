//! `list`: サービス一覧と件数を表示する

use super::{format_tags, open_db};
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
    println!("{}  Count  Tags", pad("Service", col));
    println!("{}", "-".repeat(col + 20));
    for (svc, cnt) in &services {
        let tags = db.tags_of(svc).expect("Failed to read tags");
        let line = format!("{}  {:>5}  {}", pad(svc, col), cnt, format_tags(&tags));
        println!("{}", line.trim_end());
    }
}

/// 表示幅（全角は 2）を考慮して右側を空白で埋める
fn pad(s: &str, width: usize) -> String {
    let spaces = width.saturating_sub(s.width());
    format!("{}{}", s, " ".repeat(spaces))
}
