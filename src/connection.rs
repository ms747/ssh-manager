use std::fs::{read_to_string, OpenOptions};
use std::io;
use std::io::Write;
use std::process::Command;
use std::str::FromStr;

pub struct Connection {
    pub host: String,
    pub username: String,
    pub port: u32,
    pub private_key: Option<String>,
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
    pub fn host(self, host: &str) -> Self {
        Connection {
            host: host.to_string(),
            ..self
        }
    }

    pub fn username(self, username: &str) -> Self {
        Connection {
            username: username.to_string(),
            ..self
        }
    }

    pub fn port(self, port: u32) -> Self {
        Connection { port, ..self }
    }

    pub fn private_key(self, private_key: &str) -> Result<Self, String> {
        match std::fs::canonicalize(&private_key) {
            Ok(path) => Ok(Connection {
                private_key: Some(path.to_str().unwrap().to_string()),
                ..self
            }),
            Err(_) => Err(format!("File : {} Not Found", private_key)),
        }
    }

    pub fn connect(&self) {
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

    pub fn question(question: &str) -> Result<String, String> {
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

    pub fn create_connection_from_questions() -> Result<Connection, String> {
        let host = Connection::question("Enter host : ")?;
        let username = Connection::question("Enter username : ")?;
        let port = Connection::question("Enter port (default=22) : ")?;
        let private_key = Connection::question("Enter Private Key Path (default=empty) : ")?;

        let mut default = Connection::default();
        default = default.host(&host).username(&username);

        if !port.is_empty() {
            default = default.port(port.parse::<u32>().unwrap_or(22));
        }

        if !private_key.is_empty() {
            default = default.private_key(&private_key)?;
        }

        Ok(default)
    }
}

pub fn print_list(hosts: &[Connection]) {
    for (i, host) in hosts.iter().enumerate() {
        println!("{}) {}", i, host.host);
    }
}

pub fn read_input(hosts: &[Connection]) -> Result<usize, String> {
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
        Err(err) => Err(err.to_string()),
    }
}

pub fn file_read() -> Result<(), String> {
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
            return Err(err);
        }
    };
    Ok(())
}

pub fn file_write(connection: &Connection) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("hosts.txt")
        .map_err(|e| e.to_string())?;
    writeln!(&mut file, "{}", connection.to_string()).map_err(|e| e.to_string())?;
    Ok(())
}
