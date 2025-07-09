use std::thread::{self, JoinHandle};

use crossbeam_channel::{Receiver, Sender};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Task>>,
}

impl ThreadPool {
    /// Create a new ThreadPool  
    ///
    /// The size is the number of thread in the pool.  
    ///
    /// # Panic
    /// The `new` function will panic if the size is zero.  
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = crossbeam_channel::unbounded();

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone()));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self.sender.as_ref() {
            Some(sender) => {
                sender.send(Task::new(f)).unwrap();
            }
            _ => unreachable!("Task Sender is empty"),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.workers.drain(..) {
            worker.thread.join().unwrap();
        }
    }
}

struct Worker {
    #[allow(dead_code)]
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Receiver<Task>) -> Self {
        let thread = thread::spawn(move || {
            while let Ok(task) = receiver.recv() {
                task.0();
            }
        });
        Worker { id, thread }
    }
}

struct Task(Box<dyn FnOnce() + Send + 'static>);

impl Task {
    fn new<F: FnOnce() + Send + 'static>(f: F) -> Self {
        Task(Box::new(f))
    }
}
