use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Task>,
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

        let (sender, reciever) = mpsc::channel();

        let arc_reciever = Arc::new(Mutex::new(reciever));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&arc_reciever)));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Task::new(f)).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<Receiver<Task>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let task = {
                    let lock = reciever.lock().unwrap();
                    println!("Worker {id} wating task");
                    lock.recv()
                };
                println!("Worker {id} got a task");
                match task {
                    Ok(task) => {
                        task.0();
                        println!("Worker {id} done a task");
                    }
                    Err(e) => {
                        println!("{e:?}");
                        break;
                    }
                }
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
