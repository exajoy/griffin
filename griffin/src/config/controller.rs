use notify::RecommendedWatcher;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::mpsc;

use super::{config::Config, loader::ConfigLoader, store::ConfigStore, watcher::ConfigWatcher};

/// This config manager handles loading
/// and storing configuration
/// with hot-reload capability
pub struct ConfigController {
    store: ConfigStore,
}

impl ConfigController {
    pub fn new(initial: Config) -> Self {
        Self {
            store: ConfigStore::new(initial),
        }
    }
    pub fn get(&self) -> Arc<Config> {
        self.store.get()
    }
    pub fn load_from_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let cfg = ConfigLoader::from_file(path)?;
        self.store.set(cfg);
        Ok(())
    }
    /// Watch file changes and auto-reload
    pub fn watch_file(
        &self,
        path: PathBuf,
        reload_tx: tokio::sync::mpsc::UnboundedSender<()>,
    ) -> RecommendedWatcher {
        let (tx, mut rx) = mpsc::unbounded_channel::<()>();
        let store = self.store.clone();
        let path_copy = path.clone();

        // Background tokio task to reload config when notified
        tokio::spawn(async move {
            while rx.recv().await.is_some() {
                let Ok(txt) = tokio::fs::read_to_string(&path_copy).await else {
                    return;
                };
                let Ok(cfg) = serde_yaml::from_str::<Config>(&txt) else {
                    return;
                };
                store.set(cfg);
                //  notify ListenerController
                let _ = reload_tx.send(());
            }
        });
        // watcher sends into tx
        ConfigWatcher::watch(&path, tx)
    }
}

#[tokio::test]
async fn test_hot_reload_on_file_change() {
    use tempfile::tempdir;

    // create temp directory and file
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.json");

    // write initial config
    std::fs::write(&path, r#"{ "message": "v1" }"#).unwrap();

    // create manager and load v1
    let manager = ConfigController::new(Config::with_message("init".into()));
    manager.load_from_file(&path).unwrap();

    // channel to receive reload notifications
    let (reload_tx, mut reload_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

    // start watching file
    let _watcher = manager.watch_file(path.clone(), reload_tx);

    // modify the config file to trigger reload
    std::fs::write(&path, r#"{ "message": "v2" }"#).unwrap();

    // await on reload event
    reload_rx.recv().await.expect("Watcher closed unexpectedly");

    // assert new config is active
    let cfg = manager.store.get();
    assert_eq!(cfg.message, "v2");
}
