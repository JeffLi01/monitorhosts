use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    path::Path,
};

use log::{error, trace, warn};
use serde::{Deserialize, Serialize};

pub struct Manager {
    pub hosts: Vec<HostConfig>,
    liveness: HashMap<String, PortStatus>,
    status: HashMap<(String, Port), PortStatus>,
    updated: bool,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            hosts: Vec::new(),
            liveness: HashMap::new(),
            status: HashMap::new(),
            updated: false,
        }
    }

    pub fn with_config(config: &Path) -> Option<Self> {
        if !config.exists() {
            warn!("config file '{config:?}' not exists");
            return None;
        }
        let content = match std::fs::read_to_string(config) {
            Ok(content) => content,
            Err(err) => {
                error!("failed to read file '{config:?}': {err}");
                return None;
            }
        };
        let hosts: Vec<HostConfig> = match serde_json::from_str(&content) {
            Ok(hosts) => hosts,
            Err(err) => {
                error!("failed deserialize '{config:?}': {err}");
                return None;
            }
        };
        Some(Self {
            hosts,
            liveness: HashMap::new(),
            status: HashMap::new(),
            updated: true,
        })
    }

    pub fn updated(&self) -> bool {
        self.updated
    }

    pub fn contains_host(&self, name: &str) -> bool {
        self.hosts.iter().any(|host| host.name == name)
    }

    pub fn add_host(&mut self, host: HostConfig) {
        trace!("inserting host {} to manager...", host.name);
        self.hosts.push(host);
        self.updated = true;
    }

    pub fn get_host(&self, index: usize) -> Option<&HostConfig> {
        self.hosts.get(index)
    }

    pub fn update_host(&mut self, index: usize, host: HostConfig) {
        trace!("updating host from manager...");
        self.hosts.remove(index);
        self.hosts.insert(index, host);
        self.updated = true;
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

    pub fn update(&mut self, name: String, port: Port, status: PortStatus) {
        self.status
            .entry((name, port))
            .and_modify(|value| {
                if *value != status {
                    *value = status;
                    self.updated = true;
                }
            })
            .or_insert_with(|| {
                self.updated = true;
                status
            });
    }

    pub fn update_liveness(&mut self, name: String, status: PortStatus) {
        self.liveness
            .entry(name)
            .and_modify(|value| {
                if *value != status {
                    *value = status;
                    self.updated = true;
                }
            })
            .or_insert_with(|| {
                self.updated = true;
                status
            });
    }

    pub fn capture(&mut self) -> Snapshot {
        let configs = self.hosts.clone();
        let liveness = self.liveness.clone();
        let status = self.status.clone();
        self.updated = false;
        Snapshot::new(configs, liveness, status)
    }
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub struct HostConfig {
    pub name: String,
    pub ports: BTreeMap<Port, bool>,
}

impl HostConfig {
    pub fn new(name: String, ports: BTreeMap<Port, bool>) -> Self {
        Self { name, ports }
    }

    pub fn with_all_enable(name: String) -> Self {
        let mut ports = BTreeMap::new();
        ports.insert(Port::Http, true);
        ports.insert(Port::Https, true);
        ports.insert(Port::Ssh, true);
        ports.insert(Port::Vnc, true);
        ports.insert(Port::Ipmi, true);
        Self::new(name, ports)
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Port {
    Http,
    Https,
    Ssh,
    Vnc,
    Ipmi,
}

impl Port {
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

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PortStatus {
    On,
    Off,
    Error,
}

impl std::fmt::Display for PortStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortStatus::On => write!(f, "⬤"),
            PortStatus::Off => write!(f, "◯"),
            PortStatus::Error => write!(f, "✕"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub configs: Vec<HostConfig>,
    pub liveness: HashMap<String, PortStatus>,
    pub status: HashMap<(String, Port), PortStatus>,
}

impl Snapshot {
    pub fn new(
        configs: Vec<HostConfig>,
        liveness: HashMap<String, PortStatus>,
        status: HashMap<(String, Port), PortStatus>,
    ) -> Self {
        Self {
            configs,
            liveness,
            status,
        }
    }
}
