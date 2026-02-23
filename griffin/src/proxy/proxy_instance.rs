use std::sync::Arc;
use tokio::{
    sync::{Mutex, watch},
    task::JoinHandle,
};

pub struct ProxyInstance {
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
    pub accept_conns: Arc<Mutex<Option<JoinHandle<()>>>>,

    pub listen_address: String,
    /// shutdown signal sender
    /// sending true tells the accept loop
    /// to stop accepting new connections
    pub shutdown_tx: watch::Sender<bool>,
}
