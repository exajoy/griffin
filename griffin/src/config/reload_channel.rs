use tokio::sync::mpsc;

pub struct ReloadChannel {
    pub tx: mpsc::UnboundedSender<()>,
    pub rx: mpsc::UnboundedReceiver<()>,
}
impl Default for ReloadChannel {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<()>();
        Self { tx, rx }
    }
}
