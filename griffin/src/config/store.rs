use arc_swap::ArcSwap;
use std::sync::Arc;

use super::config::Config;

#[derive(Clone)]
pub struct ConfigStore {
    /// we need to clone ConfigStore
    /// to use it in Tokio tasks,
    /// but ArcSwap does not implement Clone
    /// one of the option is to wrap it in Arc
    inner: Arc<ArcSwap<Config>>,
}

impl ConfigStore {
    pub fn new(initial: Config) -> Self {
        Self {
            inner: Arc::new(ArcSwap::from_pointee(initial)),
        }
    }
    pub fn get(&self) -> Arc<Config> {
        self.inner.load_full()
    }

    pub fn set(&self, cfg: Config) {
        self.inner.store(Arc::new(cfg));
    }
}

#[test]
fn test_atomic_swap_changes_pointer() {
    let store = ConfigStore::new(Config::with_message("A".into()));

    let old_ptr = Arc::as_ptr(&store.get()).cast::<()>();

    store.set(Config::with_message("B".into()));

    let new_ptr = Arc::as_ptr(&store.get()).cast::<()>();

    // Pointer must change
    assert_ne!(old_ptr, new_ptr);

    // Value must update
    assert_eq!(store.get().message, "B");
}

#[tokio::test]
async fn test_async_workers_see_updates() {
    use tokio::{
        task,
        time::{Duration, sleep},
    };

    let store = Arc::new(ConfigStore::new(Config::with_message("v1".into())));

    let store_task = store.clone(); // <- to move into task
    let t = task::spawn(async move {
        for _ in 0..100 {
            let cfg = store_task.get();
            assert!(cfg.message.starts_with('v'));
            tokio::task::yield_now().await;
        }
    });

    // Update config asynchronously
    store.set(Config::with_message("v2".into()));
    sleep(Duration::from_millis(1)).await;
    store.set(Config::with_message("v3".into()));
    sleep(Duration::from_millis(1)).await;

    t.await.unwrap();

    // Final check
    let cfg = store.get();
    assert_eq!(cfg.message, "v3");
}

#[test]
fn test_old_config_lives_until_dropped() {
    let store = ConfigStore::new(Config::with_message("old".into()));

    // Obtain a reader Arc BEFORE update
    let old_cfg = store.get();

    // Swap in a new config
    store.set(Config::with_message("new".into()));

    // Old cfg must still be valid and unchanged
    assert_eq!(old_cfg.message, "old");

    // New cfg is also available
    let new_cfg = store.get();
    assert_eq!(new_cfg.message, "new");
}

#[test]
fn test_concontainer_reads_are_safe() {
    use std::{sync::Arc, thread, time::Duration};

    let store = Arc::new(ConfigStore::new(Config::with_message("v1".into())));

    // Spawn reader threads
    let mut readers = vec![];

    for _ in 0..10 {
        let store = store.clone();
        readers.push(thread::spawn(move || {
            for _ in 0..1000 {
                let cfg = store.get();
                assert!(!cfg.message.is_empty());
            }
        }));
    }

    // Writer thread: update config repeatedly
    let store = store.clone();
    let writer = thread::spawn(move || {
        for i in 0..10 {
            store.set(Config::with_message(format!("v{}", i)));
            thread::sleep(Duration::from_millis(5));
        }
    });

    // every reader thread finished without panic
    for reader in readers {
        reader.join().unwrap();
    }

    // the writer finished all updates without panic
    writer.join().unwrap();
}
