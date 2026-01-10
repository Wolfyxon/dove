use tokio::sync::mpsc::{self, Sender, Receiver};

pub type MPSCChannel<T> = (mpsc::Sender<T>, mpsc::Receiver<T>);

pub const COMM_BUFFER_SIZE: usize = 512;
