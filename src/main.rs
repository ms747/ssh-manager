use ssh_manager::connection::{file_read, file_write, Connection};

enum AppState {
    AddClient,
    ConnectClient,
    GetUsage,
    NotImplemented(String),
}

fn usage() {
    println!("USAGE : ssh-manager [OPTIONS]");
    println!();
    println!("  -a, --add\t\tAdd Host");
    println!("  -h, --help\t\tHelp");
    std::process::exit(1);
}

fn arguments() -> AppState {
    let args: Vec<_> = std::env::args().collect();

    if args.len() > 2 {
        usage();
    }

    if args.len() > 1 {
        let option = &args[1][..];
        match option {
            "-a" | "--add" => AppState::AddClient,
            "-h" | "--help" => AppState::GetUsage,
            "" => AppState::ConnectClient,
            _ => AppState::NotImplemented(option.to_string()),
        }
    } else {
        AppState::ConnectClient
    }
}

fn main() -> Result<(), String> {
    match arguments() {
        AppState::AddClient => {
            let new_client = Connection::create_connection_from_questions()?;
            file_write(&new_client)?;
        }
        AppState::ConnectClient => {
            file_read()?;
        }
        AppState::GetUsage => {
            usage();
        }
        AppState::NotImplemented(cmd) => {
            println!("{} is not supported.", cmd);
        }
    }
    Ok(())
}
