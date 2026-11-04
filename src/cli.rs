//! CLI 定義（clap derive）

use clap::{ArgGroup, Parser, Subcommand};

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

Passwords are encrypted with a key stored in the OS key store
(macOS: Keychain / Windows: Credential Manager; otherwise set GENPASSWD_EX_KEY).

Run 'genpasswd_ex <COMMAND> -h' for command-specific options."
)]
pub struct Args {
    /// Password length
    #[arg(short, long, default_value_t = 16)]
    pub length: usize,

    /// Include symbols
    #[arg(short = 'k', long)]
    pub symbols: bool,

    /// Do not copy the generated password to the clipboard
    #[arg(short, long)]
    pub no_copy: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
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
