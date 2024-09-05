use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, RwLock}, thread::{self, JoinHandle}, time::Duration};

use crate::{
    manager::{Manager, Snapshot},
    ui::*,
};
use slint::*;

pub struct Monitor {
    timer: Timer,
    thread: JoinHandle<()>,
    terminate_flag: Arc<AtomicBool>,
}

impl Monitor {
    pub fn new(manager: Arc<RwLock<Manager>>, window: &MainWindow) -> Self {
        let terminate_flag = Arc::new(AtomicBool::new(false));
        let snapshot: Arc<RwLock<Option<Snapshot>>> = Arc::new(RwLock::new(None));

        let update_timer = Timer::default();
        let snapshot_clone = snapshot.clone();
        update_timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(1),
            {
                let weak_window = window.as_weak();
    
                move || {
                    let window = weak_window.clone();
                    update(window, snapshot_clone.clone());
                }
            },
        );

        let flag = terminate_flag.clone();
        let handle = thread::spawn(move || {
            loop {
                if flag.load(Ordering::Relaxed) {
                    break;
                }
                let s = manager.read().unwrap().check();
                snapshot.write().unwrap().replace(s);
                thread::sleep(Duration::from_secs(10));
            }
        });
    
        Self {
            timer: update_timer,
            thread: handle,
            terminate_flag,
        }
    }

    pub fn join(self) {
        self.timer.stop();
        self.terminate_flag.store(true, Ordering::Relaxed);
        self.thread.join().unwrap();
    }
}

fn update(window: Weak<MainWindow>, snapshot: Arc<RwLock<Option<Snapshot>>>) {
    dbg!(&snapshot);
    window
        .upgrade_in_event_loop(move |window| {
            println!("upgrade_in_event_loop");
            if snapshot.read().unwrap().is_some() {
                let s = snapshot.read().unwrap().as_ref().unwrap().to_owned();
                let model = HostsStatusModel::from(s).to_tree_view_model();
                let adapter = window.global::<MainWindowAdapter>();
                adapter.set_model(model);
            }
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
                attrs.append(&mut config
                    .ports
                    .iter()
                    .map(|(port, enabled)| {
                        if *enabled {
                            match value.status.get(&(name.clone(), *port)) {
                                Some(online) => if *online { "⬤" } else { "◯" },
                                None => "NA",
                            }
                        } else {
                            ""
                        }
                    })
                    .map(ToString::to_string)
                    .collect());
                attrs
            })
            .collect();
        HostsStatusModel {
            hosts,
        }
    }
}
