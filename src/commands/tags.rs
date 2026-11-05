//! `tags`: タグごとに、付いているサービスを一覧表示する

use super::open_db;
use std::collections::BTreeMap;

pub fn run() {
    let db = open_db();
    let mut by_tag: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (service, tag) in db.all_service_tags().expect("Failed to read tags") {
        by_tag.entry(tag).or_default().push(service);
    }
    if by_tag.is_empty() {
        println!("No tags found.");
        return;
    }
    for (i, (tag, services)) in by_tag.iter().enumerate() {
        if i > 0 {
            println!();
        }
        println!("#{} ({} service(s))", tag, services.len());
        for service in services {
            println!("  {}", service);
        }
    }
}
