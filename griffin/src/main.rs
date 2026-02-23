use clap::Parser;
use griffin::{
    args::args::Args,
    config::{controller::ConfigController, loader::ConfigLoader},
    connection::proxy_connection_handler::ProxyConnectionHandler,
    proxy::proxy_supervisor::ProxySupervisor,
};
use std::path::PathBuf;
use tokio::sync::mpsc::unbounded_channel;
use tower::BoxError;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let args = Args::parse();
    let config_path: PathBuf = args.config_path.into();

    // load initial config
    let config = ConfigLoader::from_file(&config_path)?;
    let config_controller = ConfigController::new(config.clone());

    let pch = ProxyConnectionHandler;
    // start initial listener and listener manager
    let proxy_server = ProxySupervisor::spawn_proxy_server(config, pch.clone()).await;
    // keep shutdown_tx alive using this variable
    // or else it will be drop immediately
    let _shutdown_tx = proxy_server.shutdown_tx.clone();
    let proxy_supervisor = ProxySupervisor::new(proxy_server, pch);

    // a channel to notify listener reload on config changes
    let (reload_tx, mut reload_rx) = unbounded_channel::<()>();

    // start watching config file for changes
    // keep watcher alive using this variable
    // or else it will be drop immediately
    let _watcher = config_controller.watch_file(config_path, reload_tx);

    // Task that listens for config reload events
    let reload_task = {
        tokio::spawn(async move {
            while reload_rx.recv().await.is_some() {
                let config = config_controller.get().as_ref().clone();
                println!("Config changed, reloading listener");
                println!("New config:{:?}", config);
                proxy_supervisor.reload_listener(config).await;
            }
        })
    };

    // println!(
    //     "[shutdown] sender_count={} receiver_count={}",
    //     shutdown_tx.sender_count(),
    //     shutdown_tx.receiver_count(),
    // );
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
