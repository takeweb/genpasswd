//! `tag`: サービスにタグを付ける

use super::{exit_with_error, format_tags, normalize_tags, open_db};

pub fn run(service: String, tags: Vec<String>) {
    let tags = normalize_tags(tags);
    let db = open_db();
    // サービス名の打ち間違いで孤立したタグができないよう、履歴のあるサービスに限る
    if db.count_service(&service).expect("Failed to read history") == 0 {
        exit_with_error(format!("no history found for service \"{}\".", service));
    }
    let n = db.add_tags(&service, &tags).expect("Failed to add tags");
    let current = db.tags_of(&service).expect("Failed to read tags");
    println!("Added {} tag(s) to service \"{}\": {}", n, service, format_tags(&current));
}
