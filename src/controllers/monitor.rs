use std::{
    net::IpAddr, sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    }, thread::{self, JoinHandle}, time::Duration
};
use std::net::TcpStream;

use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};
use tokio::runtime::Runtime;

use crate::{
    manager::{Manager, PortStatus, Snapshot},
    ui::*,
};
use log::{error, trace, warn};
use slint::*;

pub struct Monitor {
    threads: Vec<JoinHandle<()>>,
    terminate_flag: Arc<AtomicBool>,
}

impl Monitor {
    pub fn new(manager: Arc<RwLock<Manager>>, window: &MainWindow) -> Self {
        let terminate_flag = Arc::new(AtomicBool::new(false));
        let mut threads = Vec::new();

        let window_weak = window.as_weak();

        let flag = terminate_flag.clone();
        let mgr = manager.clone();
        threads.push(thread::spawn(move || loop {
            if flag.load(Ordering::Relaxed) {
                break;
            }
            if mgr.read().unwrap().updated() {
                update(window_weak.clone(), mgr.clone());
            }
        }));

        let flag = terminate_flag.clone();
        let mgr = manager.clone();
        threads.push(thread::spawn(move || loop {
            if flag.load(Ordering::Relaxed) {
                break;
            }
            let hosts = mgr.read().unwrap().hosts.clone();
            hosts.iter().for_each(|config| {
                config.ports.iter().for_each(|(port, enabled)| {
                    if *enabled {
                        let status = tcping(&config.name, port.u16());
                        mgr.write()
                            .unwrap()
                            .update(config.name.to_owned(), port.to_owned(), status);
                    }
                });
            });

            thread::sleep(Duration::from_secs(10));
        }));

        let flag = terminate_flag.clone();
        let mgr = manager.clone();
        let rt = Runtime::new().unwrap();
        threads.push(thread::spawn(move || {
            rt.block_on(async {
                let client_v4 = Client::new(&Config::default()).expect("surge-ping Config for IPv4 should be created successfully");
                let client_v6 = Client::new(&Config::builder().kind(ICMP::V6).build()).expect("surge-ping Config for IPv6 should be created successfully");

                loop {
                    if flag.load(Ordering::Relaxed) {
                        break;
                    }
                    let hosts = mgr.read().unwrap().hosts.clone();
                    for config in hosts {
                        let status = match config.name.parse() {
                            Ok(IpAddr::V4(addr)) => {
                                ping(&client_v4, IpAddr::V4(addr)).await
                            }
                            Ok(IpAddr::V6(addr)) => {
                                ping(&client_v6, IpAddr::V6(addr)).await
                            }
                            Err(e) => {
                                error!("{} parse to ipaddr error: {}", config.name, e);
                                PortStatus::Error
                            },
                        };
                        mgr.write()
                            .unwrap()
                            .update_liveness(config.name.to_owned(), status);
                    }

                    thread::sleep(Duration::from_secs(10));
                }
            });
        }));

        Self {
            threads,
            terminate_flag,
        }
    }

    pub fn join(self) {
        trace!("wating monitor to terminate...");
        self.terminate_flag.store(true, Ordering::Relaxed);
        self.threads
            .into_iter()
            .for_each(|thread| thread.join().unwrap());
        trace!("wating monitor to terminate...done");
    }
}

fn update(window: Weak<MainWindow>, manager: Arc<RwLock<Manager>>) {
    let snapshot = manager.write().unwrap().capture();
    window
        .upgrade_in_event_loop(move |window| {
            trace!("updating MainWindowAdapter...");
            let model = HostsStatusModel::from(snapshot).to_tree_view_model();
            let adapter = window.global::<MainWindowAdapter>();
            adapter.set_model(model);
            trace!("updating MainWindowAdapter...done");
        })
        .unwrap();
}

struct HostsStatusModel {
    hosts: Vec<Vec<String>>,
}

impl HostsStatusModel {
    fn to_tree_view_model(self) -> ModelRc<ModelRc<StandardListViewItem>> {
        let hosts: Vec<ModelRc<StandardListViewItem>> = self
            .hosts
            .into_iter()
            .map(|x| {
                let row: Vec<StandardListViewItem> = x
                    .into_iter()
                    .map(|s| {
                        let mut item = StandardListViewItem::default();
                        item.text = s.into();
                        item
                    })
                    .collect();
                ModelRc::new(VecModel::from(row))
            })
            .collect();
        ModelRc::new(VecModel::from(hosts))
    }
}

impl From<Snapshot> for HostsStatusModel {
    fn from(value: Snapshot) -> Self {
        let hosts: Vec<Vec<String>> = value
            .configs
            .iter()
            .map(|config| {
                let name = config.name.to_owned();
                let mut attrs = vec![name.clone()];
                let liveness = match value.liveness.get(&name) {
                    Some(status) => status.to_string(),
                    None => "NA".to_string(),
                };
                attrs.push(liveness);
                attrs.append(
                    &mut config
                        .ports
                        .iter()
                        .map(|(port, enabled)| {
                            if *enabled {
                                match value.status.get(&(name.clone(), *port)) {
                                    Some(status) => status.to_string(),
                                    None => "NA".to_string(),
                                }
                            } else {
                                "".to_string()
                            }
                        })
                        .collect(),
                );
                attrs
            })
            .collect();
        HostsStatusModel { hosts }
    }
}

fn tcping(host: &str, port: u16) -> PortStatus {
    let target = std::format!("{host}:{port}");
    match target.parse() {
        Ok(addr) => {
            match TcpStream::connect_timeout(&addr, Duration::from_secs(1)) {
                Ok(_) => PortStatus::On,
                Err(err) => {
                    error!("failed to connect '{target}': {err}");
                    PortStatus::Off
                }
            }
        },
        Err(err) => {
            error!("failed to parse host '{target}': {err}");
            PortStatus::Error
        },
    }
}

async fn ping(client: &Client, addr: IpAddr) -> PortStatus {
    let payload = [0; 56];
    let mut pinger = client.pinger(addr, PingIdentifier(rand::random())).await;
    pinger.timeout(Duration::from_secs(1));
    match pinger.ping(PingSequence(0), &payload).await {
        Ok(_) => PortStatus::On,
        Err(err) => {
            warn!("ping '{}' error: {}", pinger.host, err);
            PortStatus::Off
        },
    }
}