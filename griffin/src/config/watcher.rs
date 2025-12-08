use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};
use tokio::sync::mpsc;

pub struct ConfigWatcher;

impl ConfigWatcher {
    pub fn watch(path: &std::path::PathBuf, tx: mpsc::UnboundedSender<()>) -> RecommendedWatcher {
        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res
                    && matches!(event.kind, EventKind::Modify(ModifyKind::Data(_)))
                {
                    println!("{:?}", event);
                    let _ = tx.send(());
                }
            })
            .unwrap();

        watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

        watcher
    }
}
