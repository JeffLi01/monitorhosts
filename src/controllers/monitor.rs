use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    manager::{Manager, Snapshot},
    ui::*,
};
use log::trace;
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
                        let alive = ping(&config.name, port.u16());
                        mgr.write()
                            .unwrap()
                            .update(config.name.to_owned(), port.to_owned(), alive);
                    }
                });
            });

            thread::sleep(Duration::from_secs(10));
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
                attrs.append(
                    &mut config
                        .ports
                        .iter()
                        .map(|(port, enabled)| {
                            if *enabled {
                                match value.status.get(&(name.clone(), *port)) {
                                    Some(online) => {
                                        if *online {
                                            "⬤"
                                        } else {
                                            "◯"
                                        }
                                    }
                                    None => "NA",
                                }
                            } else {
                                ""
                            }
                        })
                        .map(ToString::to_string)
                        .collect(),
                );
                attrs
            })
            .collect();
        HostsStatusModel { hosts }
    }
}

use std::net::{SocketAddr, TcpStream};
fn ping<S: std::fmt::Display>(name: S, port: u16) -> bool {
    let addr: SocketAddr = std::format!("{name}:{port}")
        .parse()
        .expect("Invalid address");
    match TcpStream::connect_timeout(&addr, Duration::from_secs(1)) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error connecting: {}", e);
            false
        }
    }
}
