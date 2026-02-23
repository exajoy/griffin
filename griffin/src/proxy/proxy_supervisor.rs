use griffin_core::telemetry::metrics::Metrics;
use http::uri::Authority;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, watch};
use tower::BoxError;

use crate::config::config::Config;
use crate::connection::connection_handler::ConnectionHandler;
use crate::proxy::proxy_instance::ProxyInstance;

/// manages hot-swapping listeners
pub struct ProxySupervisor<H: ConnectionHandler + Clone> {
    ///ArcSwap allows instant pointer swap with no locks.
    pub active_proxy: arc_swap::ArcSwapOption<ProxyInstance>,
    pub connection_handler: Arc<H>,
}

impl<H: ConnectionHandler + Clone> ProxySupervisor<H> {
    pub fn new(connection_handler: H) -> Self {
        ProxySupervisor {
            active_proxy: arc_swap::ArcSwapOption::<ProxyInstance>::from(None),
            connection_handler: Arc::new(connection_handler),
        }
    }

    /// Start a TCP accept loop
    pub async fn spawn_proxy_server(
        config: Config,
        connection_handler: Arc<H>,
    ) -> Arc<ProxyInstance> {
        let listen_address = format!("{}:{}", config.listen_host, config.listen_port);
        let target_authority = format!("{}:{}", config.target_host, config.target_port);

        let listener = TcpListener::bind(listen_address.clone()).await.unwrap();

        println!("[server: {}] start listening", listen_address);

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let metrics = Arc::new(Metrics::new());
        let target_authority = Authority::from_str(&target_authority).unwrap();

        let listen_address_clone = listen_address.clone();
        let accept_conns = tokio::spawn(async move {
            loop {
                // INFO: if there is notification on shutdown_rx,
                // break the loop and stop accepting new
                // connections
                tokio::select! {
                    result = shutdown_rx.changed() => {
                        match result {
                            Ok(_) => {
                                // println!("Shutdown signal received");
                                println!("[server: {}] stop receiving requests", listen_address);
                                break;
                            }
                            Err(err) => {
                                println!("Shutdown sender dropped {}", err);
                                break;
                            }
                        }
                        // println!("[server: {}] stop receiving requests", listen_address);
                        // break;
                    }

                    accept = listener.accept() => {
                        match accept {
                            Ok(( stream, _peer)) => {
                                let connection_handler = Arc::clone(&connection_handler);
                                tokio::spawn({
                                    let metrics = metrics.clone();
                                    let target_authority = target_authority.clone();
                                    async move {
                                        connection_handler
                                            .serve_connection(stream, metrics, target_authority)
                                            .await;
                                    }
                                });
                            }
                            Err(e) => eprintln!("Accept error: {}", e),
                        }
                    }
                }
            }
            println!("[server: {}] is draining", listen_address);
        });

        Arc::new(ProxyInstance {
            accept_conns: Arc::new(Mutex::new(Some(accept_conns))),
            shutdown_tx,
            listen_address: listen_address_clone,
        })
    }

    /// Hot-reload: start new listener, drain old one
    pub async fn load_listener(&self, config: Config) -> Result<(), BoxError> {
        let maybe_old_pi = self.active_proxy.load_full();
        if let Some(old_pi) = maybe_old_pi {
            // stop the old accept loop first
            old_pi.shutdown_tx.send(true)?;
        }

        // wait until accept loop exits
        tokio::task::yield_now().await;

        let connection_handler = self.connection_handler.clone();

        // let cloned_config = config.clone();
        //  bind new listener
        let new_pi = ProxySupervisor::spawn_proxy_server(config, connection_handler).await;

        // swap pointers
        let maybe_old_pi = self.active_proxy.swap(Some(new_pi.clone()));

        if let Some(old_pi) = maybe_old_pi {
            // println!("Hot-reloading config {:#?}", cloned_config);
            let listen_address = old_pi.listen_address.clone();
            // get old task
            let accept_conns = old_pi.accept_conns.lock().await.take().unwrap();

            // resume polling old listener in another thread
            // it will not receive new connections
            // it drains existing connections gracefully
            tokio::spawn(async move {
                match accept_conns.await {
                    Ok(_) => println!("[server: {}] drained", listen_address),
                    Err(e) => println!("Failed to drain old listener: {:?}", e),
                }
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use crate::connection::connection_handler::ConnectionHandler;

    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time::Duration;
    #[derive(Clone)]
    pub struct MockStreamHandler {
        pub notify: Arc<tokio::sync::Notify>,
    }

    impl ConnectionHandler for MockStreamHandler {
        async fn serve_connection(
            &self,
            mut stream: tokio::net::TcpStream,
            _metrics: Arc<Metrics>,
            authority: Authority,
        ) {
            let notify = self.notify.clone();

            let host = authority.host();
            let port = authority.port_u16().unwrap_or(80);
            let address: SocketAddr = format!("{}:{}", host, port)
                .parse()
                .expect("invalid socket addr");
            // let listen_address = listen_address.to_string();
            let backend_id = format!("server: {}", address);
            let mut buf = [0u8; 1024];
            if let Ok(n) = stream.read(&mut buf).await {
                let text = String::from_utf8_lossy(&buf[..n]);
                println!("[{}] received string: {}", backend_id, text);
            }
            println!("[{}] will wait", backend_id);
            // Simulate long-running request
            notify.notified().await;
            println!("[{}] continues", backend_id);
            // Write response
            if let Err(e) = stream.write_all(b"hello from backend").await {
                eprintln!("write error: {:?}", e);
                return;
            }

            // Ensure all bytes are pushed out to kernel buffers
            if let Err(e) = stream.flush().await {
                eprintln!("flush error: {:?}", e);
                return;
            }

            println!("[{}] shutdown", backend_id);
            // Gracefully close write half → client receives FIN (EOF)
            let _ = stream.shutdown().await;
            stream.shutdown().await.ok();
            drop(stream);
        }
    }
    async fn get_free_port() -> Result<u16, std::io::Error> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        Ok(addr.port())
    }
    async fn spawn_real_tcp_client(
        addr: &str,
    ) -> tokio::task::JoinHandle<Result<(), std::io::Error>> {
        let addr = addr.to_string();

        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            println!("[client] connecting…");
            let mut stream = tokio::net::TcpStream::connect(addr).await?;
            println!("[client] connected");

            // Make sure server sees us as alive
            stream.write_all(b"hello from client").await?;
            println!("[client] wrote ping");

            // Wait until server finishes & shuts down write half
            let mut buf = [0u8; 1024];
            println!("[client] waiting for server");
            let n = stream.read(&mut buf).await?;
            let text = String::from_utf8_lossy(&buf[..n]);
            println!("[client] received string: {}", text);
            Ok(())
        })
    }
    #[tokio::test]
    async fn test_draining_different_ip_addresses() {
        // shared notify object used by mock handler
        let notify = Arc::new(tokio::sync::Notify::new());

        let port1 = get_free_port().await.unwrap();
        // start initial listener
        let cfg1 = Config {
            listen_host: "127.0.0.1".into(),
            listen_port: port1,
            target_host: "".into(),
            target_port: 0,
            #[cfg(test)]
            message: "".into(),
        };

        let mock_handler = MockStreamHandler {
            notify: notify.clone(),
        };
        // let initial_listener =
        //     ProxySupervisor::spawn_proxy_server(cfg1.clone(), mock_handler.clone()).await;

        let proxy_supervisor = ProxySupervisor::new(mock_handler.clone());

        proxy_supervisor.load_listener(cfg1.clone()).await;
        let address_1 = format!("{}:{}", cfg1.listen_host, cfg1.listen_port);
        let client = spawn_real_tcp_client(&address_1).await;

        // wait for request to hit the port
        tokio::time::sleep(Duration::from_millis(100)).await;

        let port2 = get_free_port().await.unwrap();
        assert!(port2 != port1, "Ports should be different for this test");
        let cfg2 = Config {
            listen_host: "127.0.0.1".into(),
            listen_port: port2, // NEW PORT!
            target_host: "".into(),
            target_port: 0,
            #[cfg(test)]
            message: "".into(),
        };
        // trigger reload

        proxy_supervisor.load_listener(cfg2.clone()).await;
        // old listener should not accept new connections
        assert!(
            TcpStream::connect(address_1).await.is_err(),
            "Old listener still accepted new connections!"
        );

        let address_2 = format!("{}:{}", cfg2.listen_host, cfg2.listen_port);
        // new listener should accept new connections
        assert!(
            TcpStream::connect(address_2).await.is_ok(),
            "New listener did not accept connections!"
        );

        tokio::time::sleep(Duration::from_millis(100)).await;

        // release in-flight request
        notify.notify_waiters();

        // client should now receive response
        // from both old and new listener
        let result = client.await.unwrap();
        assert!(
            result.is_ok(),
            "Client failed to receive response: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_draining_same_ip_address() {
        // shared notify object used by mock handler
        let notify = Arc::new(tokio::sync::Notify::new());

        let port = get_free_port().await.unwrap();
        // start initial listener
        let cfg1 = Config {
            listen_host: "127.0.0.1".into(),
            listen_port: port,
            target_host: "".into(),
            target_port: 0,
            #[cfg(test)]
            message: "".into(),
        };

        let mock_handler = MockStreamHandler {
            notify: notify.clone(),
        };
        // let initial_listener =
        //     ProxySupervisor::spawn_proxy_server(cfg1.clone(), mock_handler.clone()).await;

        let proxy_supervisor = ProxySupervisor::new(mock_handler.clone());

        proxy_supervisor.load_listener(cfg1.clone()).await;

        let address = format!("{}:{}", cfg1.listen_host, cfg1.listen_port);
        let client = spawn_real_tcp_client(&address).await;

        // wait for request to hit the port
        tokio::time::sleep(Duration::from_millis(100)).await;

        let cfg2 = Config {
            listen_host: "127.0.0.1".into(),
            listen_port: port, // SAME PORT!
            target_host: "".into(),
            target_port: 0,
            #[cfg(test)]
            message: "".into(),
        };
        // trigger reload

        proxy_supervisor.load_listener(cfg2.clone()).await;

        // listener should still accept new connections
        assert!(
            TcpStream::connect(address).await.is_ok(),
            "listener did not accept connections!"
        );

        tokio::time::sleep(Duration::from_millis(100)).await;

        // release in-flight request
        notify.notify_waiters();

        // client should now receive response
        // from both old and new listener
        let result = client.await.unwrap();
        assert!(
            result.is_ok(),
            "Client failed to receive response: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_mocking() {
        // Shared notify object used by mock handler
        let notify = Arc::new(tokio::sync::Notify::new());

        let port = get_free_port().await.unwrap();
        // Start initial listener
        let cfg = Config {
            listen_host: "127.0.0.1".into(),
            listen_port: port,
            target_host: "127.0.0.1".into(),
            target_port: 1234,
            #[cfg(test)]
            message: "".into(),
        };

        let mock_handler = MockStreamHandler {
            notify: notify.clone(),
        };
        // let initial_listener =
        //     ProxySupervisor::spawn_proxy_server(cfg.clone(), mock_handler.clone()).await;

        let proxy_supervisor = ProxySupervisor::new(mock_handler.clone());
        proxy_supervisor.load_listener(cfg.clone()).await;

        let address = format!("{}:{}", cfg.listen_host, cfg.listen_port);
        let client = spawn_real_tcp_client(&address).await;

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Release in-flight request → drain old listener
        notify.notify_waiters();

        // Client should now receive FIN, not RST
        let result = client.await.unwrap();
        assert!(
            result.is_ok(),
            "Client failed to receive response: {:?}",
            result
        );
    }
}
