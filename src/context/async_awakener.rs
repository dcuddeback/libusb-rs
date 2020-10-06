use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

type Signal = Arc<Mutex<bool>>;

pub struct AsyncAwakener {
    signal: Signal,
    _thread: JoinHandle<()>,
}

impl Drop for AsyncAwakener {
    fn drop(&mut self) {
        *self.signal.lock().unwrap() = false;
    }
}

impl AsyncAwakener {
    pub fn spawn<F: 'static + Send + FnMut(), R: 'static + Send + FnMut()>(
        mut func: F,
        mut do_after_stop: R,
    ) -> Self {
        let signal = Arc::new(Mutex::new(true));

        let thread_signal = signal.clone();
        let thread = thread::spawn(move || {
            while *thread_signal.lock().unwrap() {
                std::thread::sleep(std::time::Duration::from_millis(100));
                func();
            }
            do_after_stop();
        });

        Self {
            _thread: thread,
            signal,
        }
    }
}
