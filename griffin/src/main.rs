use clap::Parser;
use griffin::{
    args::args::Args,
    config::{loader::ConfigLoader, manager::ConfigManager},
    listener::manager::ListenerManager,
};
use std::path::PathBuf;
use tokio::sync::mpsc::unbounded_channel;
use tower::BoxError;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let args = Args::parse();
    let config_path: PathBuf = args.config_path.into();

    // load initial config
    let initial_cfg = ConfigLoader::from_file(&config_path)?;
    let cfg_manager = ConfigManager::new(initial_cfg.clone());

    // start initial listener and listener manager
    let initial_listener = ListenerManager::start_listener(initial_cfg).await;
    let listener_manager = ListenerManager::new(initial_listener);

    // a channel to notify listener reload on config changes
    let (reload_tx, mut reload_rx) = unbounded_channel::<()>();

    // start watching config file for changes
    let _watcher = cfg_manager.watch_file(config_path, reload_tx);

    // Task that listens for config reload events
    let reload_task = {
        // let cfg_manager = cfg_manager.clone();
        // let listener_manager = listener_manager.clone();

        tokio::spawn(async move {
            while reload_rx.recv().await.is_some() {
                let cfg = cfg_manager.get().as_ref().clone();
                println!("Config changed, reloading listener");
                println!("New config:{:?}", cfg);
                listener_manager.reload_listener(cfg).await;
            }
        })
    };

    //  graceful shutdown (Ctrl-C)
    println!("Griffin Proxy started. Press Ctrl+C to shut down.");

    tokio::signal::ctrl_c().await?;
    println!("Shutdown signal received. Stopping...");

    // wait for reload task to finish
    reload_task.abort(); // or handle gracefully
    let _ = reload_task.await;

    println!("Griffin Proxy shutdown complete.");
    Ok(())
}
