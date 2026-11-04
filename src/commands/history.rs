//! `history`: 履歴を表示する（`-s` 省略時は全サービスをマスク表示）

use super::{load_cipher, open_db};
use unicode_width::UnicodeWidthStr;

pub fn run(service: Option<String>) {
    let db = open_db();
    // 全サービス表示の時はパスワードをマスクする（復号しないのでキーチェーンにもアクセスしない）
    let masked = service.is_none();
    let services = match service {
        Some(service) => vec![service],
        None => db
            .list_services()
            .expect("Failed to list services")
            .into_iter()
            .map(|(svc, _)| svc)
            .collect(),
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
        print_service_header(service, entries.len());
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

/// サービス見出し（`=` の罫線で囲み、端末なら太字・色付きで表示）
fn print_service_header(service: &str, count: usize) {
    use std::io::IsTerminal;

    let title = format!("=== {} ({} record(s)) ", service, count);
    let line = format!("{}{}", title, "=".repeat(80usize.saturating_sub(title.width())));
    if std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none() {
        println!("\x1b[1;36m{}\x1b[0m", line);
    } else {
        println!("{}", line);
    }
}
