use std::{collections::{BTreeMap, HashMap}, fmt::Display, hash::Hash, time::Duration};

use log::trace;

pub struct Manager {
    hosts: Vec<HostConfig>,
}

impl Manager {
    pub fn new() -> Self {
        Self { hosts: Vec::new() }
    }

    pub fn insert_host(&mut self, host: HostConfig) {
        trace!("inserting host to manager...");
        self.hosts.push(host);
    }

    pub fn check(&self) -> Snapshot {
        let configs = self.hosts.clone();
        let mut status = HashMap::new();
        configs.iter()
            .for_each(|config| {
                config.ports.iter()
                    .for_each(|(port, enabled)| {
                        if *enabled {
                            let alive = ping(&config.name, port.u16());
                            status.insert((config.name.to_owned(), port.to_owned()), alive);
                        }
                    });
            });
        Snapshot::new(configs, status)
    }
}

#[derive(Clone, Debug)]
pub struct HostConfig {
    pub name: String,
    pub ports: BTreeMap<Port, bool>,
}

impl HostConfig {
    pub fn new(name: String, ports: BTreeMap<Port, bool>) -> Self {
        Self { name, ports }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Port {
    Http,
    Https,
    Ssh,
    Vnc,
    Ipmi,
}

impl Port {
    pub fn name(&self) -> String {
        let name = match self {
            Port::Http => "HTTP",
            Port::Https => "HTTPS",
            Port::Ssh => "SSH",
            Port::Vnc => "VNC",
            Port::Ipmi => "IPMI",
        };
        name.to_owned()
    }

    pub fn u16(&self) -> u16 {
        match self {
            Port::Http => 80,
            Port::Https => 443,
            Port::Ssh => 22,
            Port::Vnc => 5900,
            Port::Ipmi => 623,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub configs: Vec<HostConfig>,
    pub status: HashMap<(String, Port), bool>,
}

impl Snapshot {
    pub fn new(configs: Vec<HostConfig>, status: HashMap<(String, Port), bool>) -> Self {
        Self { configs, status }
    }
}

use std::net::{TcpStream, SocketAddr};
fn ping<S: Display>(name: S, port: u16) -> bool {
    let addr: SocketAddr = format!("{name}:{port}").parse().expect("Invalid address");
    match TcpStream::connect_timeout(&addr, Duration::from_secs(1)) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error connecting: {}", e);
            false
        },
    }
}