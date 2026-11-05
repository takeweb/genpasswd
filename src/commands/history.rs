//! `history`: 履歴を表示する（`-s` 省略時は全サービスをマスク表示）

use super::{format_tags, load_cipher, open_db, strip_tag};
use unicode_width::UnicodeWidthStr;

pub fn run(service: Option<String>, tag_filter: Vec<String>) {
    let db = open_db();
    // 全サービス表示の時はパスワードをマスクする（復号しないのでキーチェーンにもアクセスしない）
    let masked = service.is_none();
    let services = match service {
        Some(service) => vec![service],
        None => {
            let all: Vec<String> = db
                .list_services()
                .expect("Failed to list services")
                .into_iter()
                .map(|(svc, _)| svc)
                .collect();
            if tag_filter.is_empty() {
                all
            } else {
                let groups = parse_tag_filter(&tag_filter);
                all.into_iter()
                    .filter(|svc| {
                        let tags = db.tags_of(svc).expect("Failed to read tags");
                        matches_filter(&tags, &groups)
                    })
                    .collect()
            }
        }
    };
    if services.is_empty() {
        println!("No services found.");
    }
    let mut cipher = None;
    for (i, service) in services.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let entries = db.get_history(service).expect("Failed to read history");
        if entries.is_empty() {
            println!("No history for service \"{}\".", service);
            continue;
        }
        let tags = db.tags_of(service).expect("Failed to read tags");
        print_service_header(service, entries.len(), &tags);
        println!("{:>4}  {:<20}  {:<20}  {}", "ID", "Username", "Password", "Created At");
        println!("{}", "-".repeat(80));
        for e in &entries {
            let user = if e.username.is_empty() { "-".to_string() } else { e.username.clone() };
            let pwd = if masked {
                "********".to_string()
            } else {
                let cipher = cipher.get_or_insert_with(load_cipher);
                cipher.decrypt(&e.password).unwrap_or_else(|err| format!("<{}>", err))
            };
            println!("{:>4}  {:<20}  {:<20}  {}", e.id, user, pwd, e.created_at);
        }
    }
}

/// `-t` の指定を解析する。`-t` ごとのグループは OR、グループ内のカンマ区切りは AND
/// 例: `-t common -t nx,stg` → [["common"], ["nx", "stg"]]
fn parse_tag_filter(values: &[String]) -> Vec<Vec<String>> {
    values
        .iter()
        .map(|v| v.split(',').map(|t| strip_tag(t).to_string()).filter(|t| !t.is_empty()).collect::<Vec<_>>())
        .filter(|group| !group.is_empty())
        .collect()
}

/// サービスのタグが、いずれかのグループのタグをすべて含んでいれば true
fn matches_filter(tags: &[String], groups: &[Vec<String>]) -> bool {
    groups.iter().any(|group| group.iter().all(|t| tags.contains(t)))
}

/// サービス見出し（`=` の罫線で囲み、端末なら太字・色付きで表示）。タグがあれば `#a #b` を付ける
fn print_service_header(service: &str, count: usize, tags: &[String]) {
    use std::io::IsTerminal;

    let title = if tags.is_empty() {
        format!("=== {} ({} record(s)) ", service, count)
    } else {
        format!("=== {} ({} record(s)) {} ", service, count, format_tags(tags))
    };
    let line = format!("{}{}", title, "=".repeat(80usize.saturating_sub(title.width())));
    if std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none() {
        println!("\x1b[1;36m{}\x1b[0m", line);
    } else {
        println!("{}", line);
    }
}
