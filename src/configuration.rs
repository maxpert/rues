use std::{thread, env};
use std::sync::mpsc::channel;
use std::sync::RwLock;
use std::time::Duration;

use config::{Config, File};
use lazy_static::lazy_static;
use log::{error, info};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

lazy_static! {
    static ref CONFIG_PATH: String = env::var("CONFIG_PATH").unwrap_or_else(|_| {
        "rules.hjson".to_string()
    });
}

lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new({
        let mut settings = Config::new();
        settings.merge(File::with_name(CONFIG_PATH.as_str())).unwrap();
        settings
    });
}

fn watch() {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1)).unwrap();
    watcher.watch(CONFIG_PATH.as_str(), RecursiveMode::NonRecursive).unwrap();

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Write(_)) => {
                load_rules();
            }
            Err(e) => {
                error!("Config reload error: {}", e);
            }
            Ok(DebouncedEvent::Create(_)) => {
                load_rules();
            }
            _ => {}
        }
    }
}

fn load_rules() {
    info!("Reloading rules...");
    SETTINGS.write().unwrap().refresh().expect("Unable to load rules");
}

pub fn get_rules() -> Config {
    return SETTINGS.read().unwrap().clone();
}

pub fn init_rules() {
    load_rules();
    thread::spawn(move || { watch(); });
}