mod clipboard;
mod crypto;
mod db;
mod password;

use clap::{ArgGroup, Parser, Subcommand};
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Secure Password Generator with History",
    after_help = "Examples:
  genpasswd_ex save -s <SERVICE> -u <USERNAME> [-l <LENGTH>] [-k] [-n]
  genpasswd_ex add -s <SERVICE> '<PASSWORD>' -u <USERNAME>
  genpasswd_ex history [-s <SERVICE>]
  genpasswd_ex copy <ID>
  genpasswd_ex rename -s <SERVICE> <NEW_SERVICE>
  genpasswd_ex remove <ID>
  genpasswd_ex delete -s <SERVICE>
  genpasswd_ex update <ID> [-u <USERNAME>] [-p '<PASSWORD>']

Passwords are encrypted with a key stored in the macOS keychain.

Run 'genpasswd_ex <COMMAND> -h' for command-specific options."
)]
struct Args {
    /// Password length
    #[arg(short, long, default_value_t = 16)]
    length: usize,

    /// Include symbols
    #[arg(short = 'k', long)]
    symbols: bool,

    /// Do not copy the generated password to the clipboard
    #[arg(short, long)]
    no_copy: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a password and save to history (-s <SERVICE> [-u <USERNAME>])
    Save {
        /// Service name
        #[arg(short, long)]
        service: String,
        /// Username for the service (defaults to the latest one in history)
        #[arg(short, long, default_value = "")]
        username: String,
        /// Password length
        #[arg(short, long, default_value_t = 16)]
        length: usize,
        /// Include symbols
        #[arg(short = 'k', long)]
        symbols: bool,
        /// Do not copy the generated password to the clipboard
        #[arg(short, long)]
        no_copy: bool,
    },
    /// Show password history for a service (all services with masked passwords if -s is omitted)
    History {
        /// Service name (omit to show all services; passwords are masked)
        #[arg(short, long)]
        service: Option<String>,
    },
    /// Copy the password of a history entry to the clipboard
    Copy {
        /// Entry ID (shown in `history`)
        id: i64,
    },
    /// List all services with saved passwords
    List,
    /// Add an existing password to history (-s <SERVICE> <PASSWORD> [-u <USERNAME>])
    #[command(alias = "register")]
    Add {
        /// Service name
        #[arg(short, long)]
        service: String,
        /// Password to add
        password: String,
        /// Username for the service (defaults to the latest one in history)
        #[arg(short, long, default_value = "")]
        username: String,
    },
    /// Update username and/or password of a history entry (-u <USERNAME> / -p <PASSWORD>)
    #[command(group(
        ArgGroup::new("fields").required(true).multiple(true).args(["username", "password"])
    ))]
    Update {
        /// Entry ID (shown in `history`)
        id: i64,
        /// New username
        #[arg(short, long)]
        username: Option<String>,
        /// New password
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Rename a service (-s <SERVICE> <NEW_SERVICE>)
    Rename {
        /// Current service name
        #[arg(short, long)]
        service: String,
        /// New service name
        new_service: String,
        /// Allow renaming into an existing service (merges the histories)
        #[arg(long)]
        merge: bool,
    },
    /// Encrypt plaintext passwords saved by older versions
    Encrypt,
    /// Remove a single history entry by ID
    Remove {
        /// Entry ID (shown in `history`)
        id: i64,
    },
    /// Delete all history for a service (-s <SERVICE>)
    Delete {
        /// Service name
        #[arg(short, long)]
        service: String,
    },
}

fn pad(s: &str, width: usize) -> String {
    let display_width = s.width();
    let spaces = width.saturating_sub(display_width);
    format!("{}{}", s, " ".repeat(spaces))
}

/// history のサービス見出し（`=` の罫線で囲み、端末なら太字・色付きで表示）
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

/// ユーザ名が省略された場合、同じサービスの履歴にある最新のユーザ名を使う
fn resolve_username(db: &db::Db, service: &str, username: String) -> String {
    if !username.is_empty() {
        return username;
    }
    db.latest_username(service)
        .expect("Failed to read history")
        .unwrap_or_default()
}

/// 生成したパスワードをクリップボードにコピーする（失敗しても処理は続行）
fn copy_to_clipboard(pwd: &str) {
    match clipboard::copy(pwd) {
        Ok(()) => eprintln!("Copied to clipboard."),
        Err(e) => eprintln!("Warning: failed to copy to clipboard ({}).", e),
    }
}

/// 暗号化鍵を読み込む（失敗したら終了）
fn load_cipher() -> crypto::Cipher {
    crypto::Cipher::load().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    })
}

/// データディレクトリ（Windows: `%APPDATA%\genpasswd_ex` / それ以外: `~/.local/share/genpasswd_ex`）
#[cfg(windows)]
fn data_dir() -> PathBuf {
    dirs::data_dir().expect("Cannot determine data directory").join("genpasswd_ex")
}

#[cfg(not(windows))]
fn data_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Cannot determine home directory");
    home.join(".local/share/genpasswd_ex")
}

fn db_path() -> PathBuf {
    if let Ok(path) = std::env::var("GENPASSWD_EX_DB") {
        return PathBuf::from(path);
    }

    let dir = data_dir();
    std::fs::create_dir_all(&dir).expect("Failed to create data directory");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .expect("Failed to set permissions of data directory");
    }
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("history_dev.db")
    } else {
        dir.join("history.db")
    }
}

fn main() {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    match args.command {
        None => {
            if args.length < 4 {
                eprintln!("Error: password length must be at least 4.");
                std::process::exit(1);
            }
            let pwd = password::generate_password(args.length, args.symbols);
            println!("Generated password: {}", pwd);
            if !args.no_copy {
                copy_to_clipboard(&pwd);
            }
        }

        Some(Command::Save { service, username, length, symbols, no_copy }) => {
            if length < 4 {
                eprintln!("Error: password length must be at least 4.");
                std::process::exit(1);
            }
            let pwd = password::generate_password(length, symbols);
            println!("Generated password: {}", pwd);
            if !no_copy {
                copy_to_clipboard(&pwd);
            }

            let cipher = load_cipher();
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let username = resolve_username(&db, &service, username);
            db.save(&service, &username, &cipher.encrypt(&pwd)).expect("Failed to save password");
            if username.is_empty() {
                eprintln!("Saved to history for service \"{}\".", service);
            } else {
                eprintln!("Saved to history for service \"{}\" (user: {}).", service, username);
            }
        }

        Some(Command::Add { service, password, username }) => {
            let cipher = load_cipher();
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let username = resolve_username(&db, &service, username);
            db.save(&service, &username, &cipher.encrypt(&password)).expect("Failed to save password");
            if username.is_empty() {
                println!("Added to history for service \"{}\".", service);
            } else {
                println!("Added to history for service \"{}\" (user: {}).", service, username);
            }
        }

        Some(Command::History { service }) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
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

        Some(Command::Copy { id }) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let Some((service, username, stored)) = db.get_entry(id).expect("Failed to read history") else {
                eprintln!("Error: no history entry with ID {}.", id);
                std::process::exit(1);
            };
            let pwd = load_cipher().decrypt(&stored).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            if let Err(e) = clipboard::copy(&pwd) {
                eprintln!("Error: failed to copy to clipboard ({}).", e);
                std::process::exit(1);
            }
            if username.is_empty() {
                println!("Copied password of ID {} (service \"{}\") to clipboard.", id, service);
            } else {
                println!("Copied password of ID {} (service \"{}\", user: {}) to clipboard.", id, service, username);
            }
        }

        Some(Command::List) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let services = db.list_services().expect("Failed to list services");
            if services.is_empty() {
                println!("No services found.");
            } else {
                let col = services.iter()
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
        }

        Some(Command::Update { id, username, password }) => {
            let password = password.map(|p| load_cipher().encrypt(&p));
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let n = db
                .update_entry(id, username.as_deref(), password.as_deref())
                .expect("Failed to update history");
            if n == 0 {
                eprintln!("Error: no history entry with ID {}.", id);
                std::process::exit(1);
            }
            println!("Updated history entry ID {}.", id);
        }

        Some(Command::Rename { service, new_service, merge }) => {
            if service == new_service {
                eprintln!("Error: new service name is the same as the current one.");
                std::process::exit(1);
            }
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let existing = db.count_service(&new_service).expect("Failed to read history");
            if existing > 0 && !merge {
                eprintln!(
                    "Error: service \"{}\" already has {} record(s). Use --merge to merge the histories.",
                    new_service, existing
                );
                std::process::exit(1);
            }
            let n = db.rename_service(&service, &new_service).expect("Failed to rename service");
            if n == 0 {
                eprintln!("Error: no history found for service \"{}\".", service);
                std::process::exit(1);
            }
            println!("Renamed service \"{}\" to \"{}\" ({} record(s)).", service, new_service, n);
        }

        Some(Command::Encrypt) => {
            let cipher = load_cipher();
            let mut db = db::Db::open(&db_path()).expect("Failed to open database");
            let rows: Vec<(i64, String)> = db
                .all_passwords()
                .expect("Failed to read history")
                .into_iter()
                .filter(|(_, pwd)| !crypto::is_encrypted(pwd))
                .map(|(id, pwd)| (id, cipher.encrypt(&pwd)))
                .collect();
            if rows.is_empty() {
                println!("All passwords are already encrypted.");
            } else {
                db.replace_passwords(&rows).expect("Failed to encrypt passwords");
                println!("Encrypted {} password(s).", rows.len());
            }
        }

        Some(Command::Remove { id }) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            match db.delete_entry(id).expect("Failed to delete history") {
                Some(service) => println!("Removed history entry ID {} (service \"{}\").", id, service),
                None => {
                    eprintln!("Error: no history entry with ID {}.", id);
                    std::process::exit(1);
                }
            }
        }

        Some(Command::Delete { service }) => {
            let db = db::Db::open(&db_path()).expect("Failed to open database");
            let n = db.delete_service(&service).expect("Failed to delete history");
            if n == 0 {
                println!("No history found for service \"{}\".", service);
            } else {
                println!("Deleted {} record(s) for service \"{}\".", n, service);
            }
        }
    }
}
