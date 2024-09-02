use std::sync::{Arc, RwLock};

use crate::{
    manager::{Manager, Snapshot},
    ui::*,
};
use slint::*;

pub fn setup(manager: Arc<RwLock<Manager>>, window: &MainWindow) -> Timer {
    let update_timer = Timer::default();
    update_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(10),
        {
            let weak_window = window.as_weak();

            move || {
                let window = weak_window.clone();
                update(manager.clone(), window);
            }
        },
    );

    update_timer
}

fn update(manager: Arc<RwLock<Manager>>, window: Weak<MainWindow>) {
    let snapshot = manager.read().unwrap().check();
    dbg!(&snapshot);
    window
        .upgrade_in_event_loop(move |window| {
            println!("upgrade_in_event_loop");
            let adapter = window.global::<MainWindowAdapter>();
            adapter.set_model(HostsStatusModel::from(snapshot).to_tree_view_model());
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
