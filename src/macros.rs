use crate::connection::Connection;

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

macro_rules! HostList {
    ($($connection:expr),+) => {{
        let mut hosts: Vec<Option<Connection>> = Vec::new();
        $(hosts.push($connection);)+
        let filtered : Vec<Connection> = hosts.into_iter().filter_map(|e| e).collect();
        filtered
    }};
}
