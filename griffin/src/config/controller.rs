use crate::config::reload_channel::ReloadChannel;

use super::{config::Config, store::ConfigStore};
use anyhow::Result;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::mpsc;
use tower::BoxError;

/// This config manager handles loading
/// and storing configuration
/// with hot-reload capability
pub struct ConfigController {
    pub store: ConfigStore,
    pub reload: ReloadChannel,
    pub watcher: Option<RecommendedWatcher>,
}

impl ConfigController {
    pub fn new(initial: Config) -> Self {
        Self {
            store: ConfigStore::new(initial),
            reload: ReloadChannel::default(),
            watcher: None,
        }
    }

    /// Watch file changes and auto-reload
    pub fn watch_file(&mut self, path: PathBuf) -> Result<(), BoxError> {
        let (file_change_tx, mut file_change_rx) = mpsc::unbounded_channel::<()>();
        let store = self.store.clone();
        let path_copy = path.clone();

        let reload_tx = self.reload.tx.clone();

        // Background tokio task to reload config when notified
        tokio::spawn(async move {
            let ok = Ok::<(), BoxError>(());
            while file_change_rx.recv().await.is_some() {
                // TODO: Maybe we just need to
                // print error here and continue
                // instead of return error
                // because it will close the loop
                let txt = tokio::fs::read_to_string(&path_copy).await?;
                let cfg = serde_yaml::from_str::<Config>(&txt)?;
                let current_config = store.get();
                if Arc::new(cfg.clone()) == current_config {
                    // no change, skip reload
                    continue;
                }
                store.set(cfg);
                // notify reload to use new config
                reload_tx.send(())?;
            }
            ok
        });

        // watch file content changes
        // NOTE: that we cannot call tokio inside this callback,
        // so we just send a signal to the
        // background task
        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res
                    && matches!(event.kind, EventKind::Modify(ModifyKind::Data(_)))
                {
                    let _ = file_change_tx.send(());
                }
            })?;

        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        // INFO: when when assign new watcher
        // the old watcher and old variables
        // related to it also get dropped
        self.watcher = Some(watcher);
        Ok(())
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
    let mut manager = ConfigController::new(Config::with_message("init".into()));
    let config = Config::from_file(&path).unwrap();
    manager.store.set(config);

    // start watching file
    manager.watch_file(path.clone()).unwrap();

    // modify the config file to trigger reload
    std::fs::write(&path, r#"{ "message": "v2" }"#).unwrap();

    // await on reload event
    manager
        .reload
        .rx
        .recv()
        .await
        .expect("Watcher closed unexpectedly");

    // assert new config is active
    let cfg = manager.store.get();
    assert_eq!(cfg.message, "v2");
}
