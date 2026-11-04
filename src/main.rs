mod cli;
mod clipboard;
mod commands;
mod crypto;
mod db;
mod password;

use clap::Parser;
use cli::{Args, Command};

fn main() {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    match args.command {
        None => commands::generate::run(args.length, args.symbols, args.no_copy),
        Some(Command::Save { service, username, length, symbols, no_copy }) => {
            commands::save::run(service, username, length, symbols, no_copy)
        }
        Some(Command::Add { service, password, username }) => commands::add::run(service, password, username),
        Some(Command::History { service }) => commands::history::run(service),
        Some(Command::Copy { id }) => commands::copy::run(id),
        Some(Command::List) => commands::list::run(),
        Some(Command::Update { id, username, password }) => commands::update::run(id, username, password),
        Some(Command::Rename { service, new_service, merge }) => commands::rename::run(service, new_service, merge),
        Some(Command::Encrypt) => commands::encrypt::run(),
        Some(Command::Remove { id }) => commands::remove::run(id),
        Some(Command::Delete { service }) => commands::delete::run(service),
    }
}
