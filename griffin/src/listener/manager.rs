use griffin_core::telemetry::metrics::Metrics;
use http::uri::Authority;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, watch};
use tokio::task::JoinHandle;

use crate::config::config::Config;
use crate::run_proxy;

pub struct ListenerHandle {
    /// running async task
    /// which is a spawned accept loop
    /// wrap with Arc allows to clone the handle

    /// Design decidion:
    /// 1.use `pub task: JoinHandle<()>`
    /// this does not allod to work with ArcSwap
    ///
    ///
    /// 2. use `pub task: Arc<JoinHandle<()>>`
    /// to get value because cloning an Arc
    /// does not move its inner value.
    /// but we can’t consume something inside an Arc.
    ///
    /// in summary we can:
    ///- check .is_finished()
    ///- keep the task alive
    ///- store it in a vector
    ///
    /// But we cannot:
    ///- .await it
    ///- get its result
    ///- ensure you wait for it to finish
    ///
    ///
    /// 3. wrap more with Mutex<Option<>>
    /// lets us:
    ///- get full ownership
    ///- call .await
    ///- get result
    ///- guarantee complete flush
    pub task: Arc<Mutex<Option<JoinHandle<()>>>>,

    /// shutdown signal sender
    /// sending true tells the accept loop
    /// to stop accepting new connections
    pub shutdown_tx: watch::Sender<bool>,
}
/// manages hot-swapping listeners
pub struct ListenerManager {
    ///ArcSwap allows instant pointer swap with no locks.
    pub current: arc_swap::ArcSwap<ListenerHandle>,
}

impl ListenerManager {
    pub fn new(initial: Arc<ListenerHandle>) -> Self {
        ListenerManager {
            current: arc_swap::ArcSwap::from(initial),
        }
    }

    /// Start a TCP accept loop
    pub async fn start_listener(config: Config) -> Arc<ListenerHandle> {
        let proxy_address = format!("{}:{}", config.proxy_host, config.proxy_port);
        let forward_authority = format!("{}:{}", config.forward_host, config.forward_port);

        let listener = TcpListener::bind(proxy_address.clone()).await.unwrap();

        println!("Listening on {}", proxy_address);

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let metrics = Arc::new(Metrics::new());
        let forward_authority = Authority::from_str(&forward_authority).unwrap();

        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        println!("Stopping accept loop for address {}", proxy_address);
                        break; // stop accepting new connections
                    }

                    accept = listener.accept() => {
                        match accept {
                            Ok(( stream, peer)) => {
                                println!("Accepted from {:?}", peer);
                                run_proxy(stream, metrics.clone(), forward_authority.clone());
                            }
                            Err(e) => eprintln!("Accept error: {}", e),
                        }
                    }
                }
            }
            println!("Listener on {} is now draining...", proxy_address);
        });

        Arc::new(ListenerHandle {
            task: Arc::new(Mutex::new(Some(task))),
            shutdown_tx,
        })
    }

    /// Hot-reload: start new listener, drain old one
    pub async fn reload_listener(&self, config: Config) {
        println!("Hot-reloading listener {:?}", config);

        let old = self.current.load_full();
        // stop the old accept loop first
        let _ = old.shutdown_tx.send(true);

        // wait until accept loop exits
        tokio::task::yield_now().await;

        //  bind new listener
        let new = ListenerManager::start_listener(config).await;

        // swap pointers
        let old = self.current.swap(new.clone());

        // get old task
        let old_task = old.task.lock().await.take().unwrap();

        println!("Old listener is draining.");
        // execute listener
        // to background task
        // avoids blocking the reload flow
        tokio::spawn(async move {
            match old_task.await {
                Ok(_) => println!("Old listener drained"),
                Err(e) => println!("Failed to drain old listener: {:?}", e),
            }
        });
    }
}
