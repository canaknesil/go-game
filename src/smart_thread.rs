use std::thread::{self, JoinHandle};
use std::sync::mpsc::{Sender, Receiver, channel};


// Thread that quits when dropped rather than being detached. It is
// the users responsability to block on receiver for the kill signal
// and end the thread's execution in a timely manner, otherwise drop
// will block.


pub struct SmartHandle {
    join_handle: Option<JoinHandle<Result<(), String>>>,
    kill_signal_tx: Sender<()>,
}


pub fn spawn<F>(f: F) -> SmartHandle
where
    F: FnOnce(Receiver<()>) -> Result<(), String> + Send + 'static,
{
    let (tx, rx) = channel();

    SmartHandle {
	join_handle: Some(thread::spawn(move || {
	    f(rx)
	})),
	kill_signal_tx: tx,
    }
}


impl Drop for SmartHandle {
    fn drop(&mut self) {
	let _ = self.kill_signal_tx.send(());
	if let Some(h) = self.join_handle.take() {
	    let _ = h.join();
	}
    }
}

