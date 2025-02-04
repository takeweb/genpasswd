use clap::Parser;

#[derive(Parser)]
#[command(author, version, about = "Secure Password Generator")]
struct Args {
    /// Length of the password
    #[arg(short, long, default_value_t = 16)]
    length: usize,

    /// Include symbols in the password
    #[arg(short, long)]
    symbols: bool,
}

fn main() {
    let args = Args::parse();

    if args.length < 4 {
        eprintln!("Password length should be at least 4 characters.");
        std::process::exit(1);
    }

    let password = genpasswd::generate_password(args.length, args.symbols);
    println!("Generated password: {}", password);
}
