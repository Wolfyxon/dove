use tokio::sync::mpsc::{self};

pub type MPSCChannel<T> = (mpsc::Sender<T>, mpsc::Receiver<T>);

pub const COMM_BUFFER_SIZE: usize = 512;
