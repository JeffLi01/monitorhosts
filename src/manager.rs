use std::{collections::{BTreeMap, HashMap}, hash::Hash};

use log::trace;

pub struct Manager {
    pub hosts: Vec<HostConfig>,
    status: HashMap<(String, Port), bool>,
    updated: bool,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            hosts: Vec::new(),
            status: HashMap::new(),
            updated: false,
        }
    }

    pub fn updated(&self) -> bool {
        self.updated
    }

    pub fn contains_host(&self, name: &str) -> bool {
        self.hosts
            .iter()
            .find(|host| host.name == name)
            .is_some()
    }

    pub fn add_host(&mut self, host: HostConfig) {
        trace!("inserting host to manager...");
        self.hosts.push(host);
        self.updated = true;
    }

    pub fn get_host(&self, index: usize) -> Option<&HostConfig> {
        self.hosts.get(index)
    }

    pub fn remove_host(&mut self, index: usize) {
        trace!("removing host from manager...");
        self.hosts.remove(index);
        self.updated = true;
    }

    pub fn clear_host(&mut self) {
        trace!("clearing hosts from manager...");
        self.hosts.clear();
        self.updated = true;
    }

    pub fn update(&mut self, name: String, port: Port, online: bool) {
        self.status.entry((name, port))
            .and_modify(|value| {
                if *value != online {
                    *value = online;
                    self.updated = true;
                }
            })
            .or_insert_with(|| {
                self.updated = true;
                online
            });
    }

    pub fn capture(&mut self) -> Snapshot {
        let configs = self.hosts.clone();
        let status = self.status.clone();
        self.updated = false;
        Snapshot::new(configs, status)
    }
}

#[derive(Clone, Debug, Hash)]
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
