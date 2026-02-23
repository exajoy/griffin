use clap::Parser;
use griffin::{
    args::args::Args,
    config::{config::Config, controller::ConfigController},
    connection::proxy_connection_handler::ProxyConnectionHandler,
    proxy::proxy_supervisor::ProxySupervisor,
};
use std::path::PathBuf;
use tower::BoxError;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let args = Args::parse();
    let config_path: PathBuf = args.config_path.into();

    // load initial config
    let config = Config::from_file(&config_path)?;
    let mut config_controller = ConfigController::new(config.clone());

    // start watching config file for changes
    // keep watcher alive using this variable
    // or else it will be drop immediately
    config_controller.watch_file(config_path)?;

    let pch = ProxyConnectionHandler;
    let proxy_supervisor = ProxySupervisor::new(pch);
    proxy_supervisor.load_listener(config).await?;

    let config_store = config_controller.store.clone();

    // Task that listens for config reload events
    let on_config_change = {
        tokio::spawn(async move {
            while config_controller.reload.rx.recv().await.is_some() {
                let config = config_store.get().as_ref().clone();
                println!("Config changed, reloading listener");
                // println!("New config:{:?}", config);
                proxy_supervisor.load_listener(config).await?;
            }
            Ok::<(), BoxError>(())
        })
    };

    println!("Griffin Proxy started. Press Ctrl+C to shut down.");

    tokio::signal::ctrl_c().await?;
    println!("Shutdown signal received. Stopping...");

    // wait for reload task to finish
    on_config_change.abort(); // or handle gracefully
    let _ = on_config_change.await;

    println!("Griffin Proxy shutdown complete.");
    Ok(())
}
