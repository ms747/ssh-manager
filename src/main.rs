use std::fs::{read_to_string, OpenOptions};
use std::io;
use std::io::Write;
use std::process::Command;
use std::str::FromStr;

#[allow(unused_macros)]
macro_rules! Host {
    ($host:expr,$username:expr) => {
        Some(Connection::new($host, $username))
    };

    ($host:expr,$username:expr,$port:expr) => {
        Some(Connection::new($host, $username).port($port))
    };

    ($host:expr,$username:expr,$port:expr,$private_key:expr) => {{
        if let Ok(path) = std::fs::canonicalize($private_key) {
            Some(
                Connection::new($host, $username)
                    .port($port)
                    .private_key(path.to_str().unwrap()),
            )
        } else {
            println!("Unable to parse {}", $host);
            None
        }
    }};
}

#[allow(unused_macros)]
macro_rules! HostList {
    ($($connection:expr),+) => {{
        let mut hosts: Vec<Option<Connection>> = Vec::new();
        $(hosts.push($connection);)+
        let filtered :Vec<Connection> = hosts.into_iter().filter_map(|e| e).collect();
        filtered
    }};
}

struct Connection {
    host: String,
    username: String,
    port: u32,
    private_key: Option<String>,
}

impl Default for Connection {
    fn default() -> Self {
        Connection {
            host: String::new(),
            username: String::new(),
            port: 22,
            private_key: None,
        }
    }
}

impl FromStr for Connection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed: String = s.chars().filter(|e| !e.is_whitespace()).collect();
        let trimmed: Vec<_> = trimmed.split('|').collect();

        let parsing_components = vec!["username", "host", "port", "private_key"];

        let mut default_connection = Connection::default();

        for value in trimmed.iter() {
            for component in parsing_components.iter() {
                let result = value.contains(component);
                match *component {
                    "username" if result => {
                        let username = value.split(':').nth(1).unwrap();
                        default_connection = Connection {
                            username: username.to_string(),
                            ..default_connection
                        };
                        break;
                    }
                    "host" if result => {
                        let host = value.split(':').nth(1).unwrap();
                        default_connection = Connection {
                            host: host.to_string(),
                            ..default_connection
                        };
                        break;
                    }
                    "port" if result => {
                        let port = value.split(':').nth(1).unwrap();
                        default_connection = Connection {
                            port: port.parse().unwrap(),
                            ..default_connection
                        };
                        break;
                    }
                    "private_key" if result => {
                        let private_key = value.split(':').nth(1).unwrap();
                        default_connection = Connection {
                            private_key: Some(private_key.to_string()),
                            ..default_connection
                        };
                        break;
                    }
                    _ => {}
                };
            }
        }

        if default_connection.host == String::new() && default_connection.username == String::new()
        {
            return Err("Username & Host are required".to_string());
        }

        Ok(default_connection)
    }
}

impl ToString for Connection {
    fn to_string(&self) -> String {
        let mut deserialize = Vec::new();

        if !self.username.is_empty() {
            deserialize.push(format!("username:{}", self.username));
        }

        if !self.host.is_empty() {
            deserialize.push(format!("host:{}", self.host));
        }

        if self.port != 22 {
            deserialize.push(format!("port:{}", self.port));
        }

        if let Some(private_key) = &self.private_key {
            deserialize.push(format!("private_key:{}", private_key));
        }

        deserialize.join("|")
    }
}

impl Connection {
    fn host(self, host: &str) -> Self {
        Connection {
            host: host.to_string(),
            ..self
        }
    }

    fn username(self, username: &str) -> Self {
        Connection {
            username: username.to_string(),
            ..self
        }
    }

    fn port(self, port: u32) -> Self {
        Connection { port, ..self }
    }

    fn private_key(self, private_key: &str) -> Result<Self, String> {
        match std::fs::canonicalize(&private_key) {
            Ok(path) => Ok(Connection {
                private_key: Some(path.to_str().unwrap().to_string()),
                ..self
            }),
            Err(_) => return Err(format!("File : {} Not Found", private_key)),
        }
    }

    fn connect(&self) {
        let mut ssh_command = String::new();

        if let Some(private_key) = &self.private_key {
            ssh_command.clear();
            ssh_command.push_str(&format!(
                "ssh -i {} {}@{} -p {}",
                private_key, self.username, self.host, self.port,
            ));
        } else {
            ssh_command.clear();
            ssh_command.push_str(&format!(
                "ssh {}@{} -p {}",
                self.username, self.host, self.port
            ));
        }

        Command::new("sh")
            .arg("-c")
            .arg(&ssh_command)
            .spawn()
            .expect("Could not spawn child process")
            .wait_with_output()
            .expect("Error waiting for output of child process");
    }

    fn question(question: &str) -> Result<String, String> {
        print!("{}", question);
        io::stdout().flush().map_err(|e| e.to_string())?;
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                return Ok(input.trim().to_string());
            }
            Err(_) => Err("Error while reading the input".to_string()),
        }
    }

    fn create_connection_from_questions() -> Result<Connection, String> {
        let host = Connection::question("Enter host : ")?;
        let username = Connection::question("Enter username : ")?;
        let port = Connection::question("Enter port (default=22) : ")?;
        let private_key = Connection::question("Enter Private Key Path (default=empty) : ")?;

        let mut default = Connection::default();
        default = default.host(&host).username(&username);

        if !port.is_empty() {
            default = default.port(port.parse::<u32>().unwrap_or_else(|_| 22));
        }

        if !private_key.is_empty() {
            default = default.private_key(&private_key)?;
        }

        Ok(default)
    }
}

fn print_list(hosts: &[Connection]) {
    for (i, host) in hosts.iter().enumerate() {
        println!("{}) {}", i, host.host);
    }
}

fn read_input(hosts: &[Connection]) -> Result<usize, String> {
    print!("Select : ");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            if let Ok(num) = input.trim().parse::<usize>() {
                if num > hosts.len() {
                    return Err("Invalid Option".to_string());
                }
                Ok(num)
            } else {
                Err("Not a number".to_string())
            }
        }
        Err(err) => return Err(err.to_string()),
    }
}

fn file_read() -> Result<(), String> {
    let contents = read_to_string("hosts.txt").map_err(|e| e.to_string())?;
    let mut hosts: Vec<Connection> = Vec::new();

    for line in contents.lines() {
        let connection = Connection::from_str(&line)?;
        hosts.push(connection);
    }

    print_list(&hosts);

    match read_input(&hosts) {
        Ok(index) => {
            hosts[index].connect();
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };
    Ok(())
}

fn file_write(connection: &Connection) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("hosts.txt")
        .map_err(|e| e.to_string())?;
    writeln!(&mut file, "{}", connection.to_string()).map_err(|e| e.to_string())?;
    Ok(())
}

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
            eprintln!("{} is not supported.", cmd);
        }
    }
    Ok(())
}
