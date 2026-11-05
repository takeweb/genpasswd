//! `untag`: サービスからタグを外す

use super::{format_tags, normalize_tags, open_db};

pub fn run(service: String, tags: Vec<String>) {
    let tags = normalize_tags(tags);
    let db = open_db();
    let n = db.remove_tags(&service, &tags).expect("Failed to remove tags");
    let current = db.tags_of(&service).expect("Failed to read tags");
    if current.is_empty() {
        println!("Removed {} tag(s) from service \"{}\" (no tags left).", n, service);
    } else {
        println!("Removed {} tag(s) from service \"{}\": {}", n, service, format_tags(&current));
    }
}
