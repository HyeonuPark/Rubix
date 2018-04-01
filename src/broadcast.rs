use std::collections::VecDeque;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

use futures::prelude::*;
use futures::task;

#[derive(Debug)]
struct Shared<T: Clone> {
    queue: VecDeque<Bucket<T>>,
    capacity: usize,
    tx_blocked: VecDeque<task::Task>,
    rx_blocked: VecDeque<task::Task>,
}

#[derive(Debug)]
struct Bucket<T> {
    data: T,
    view_count: usize,
}

impl<T> Bucket<T> {
    fn new(data: T) -> Self {
        Bucket { data, view_count: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct Sender<T: Clone>(Weak<RefCell<Shared<T>>>);

#[derive(Debug)]
pub struct SendError<T>(T);

#[derive(Debug, Clone)]
pub struct Receiver<T: Clone>(Rc<RefCell<Shared<T>>>);

// Receiver never fail
#[derive(Debug)]
pub enum ReceiveError {}

pub fn channel<T: Clone>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let shared = Rc::new(RefCell::new(Shared::new(capacity)));

    (Sender(Rc::downgrade(&shared)), Receiver(shared))
}

impl<T: Clone> Shared<T> {
    fn new(capacity: usize) -> Self {
        Shared {
            queue: Default::default(),
            capacity,
            rx_blocked: Default::default(),
            tx_blocked: Default::default(),
        }
    }
}

impl<T: Clone> Sink for Sender<T> {
    type SinkItem = T;
    type SinkError = SendError<T>;

    fn start_send(&mut self, item: T) -> StartSend<T, SendError<T>> {
        match self.0.upgrade() {
            None => Err(SendError(item)),
            Some(shared) => Ok({
                let mut shared = shared.borrow_mut();

                if shared.queue.len() >= shared.capacity {
                    shared.tx_blocked.push_back(task::current());
                    AsyncSink::NotReady(item)
                } else {
                    shared.queue.push_back(Bucket::new(item));

                    for rx in shared.rx_blocked.drain(..) {
                        rx.notify();
                    }

                    AsyncSink::Ready
                }
            })
        }
    }

    fn poll_complete(&mut self) -> Poll<(), SendError<T>> {
        Ok(Async::Ready(()))
    }

    fn close(&mut self) -> Poll<(), SendError<T>> {
        Ok(Async::Ready(()))
    }
}

impl<T: Clone> Stream for Receiver<T> {
    type Item = T;
    type Error = ReceiveError;

    fn poll(&mut self) -> Poll<Option<T>, ReceiveError> {
        if Rc::weak_count(&self.0) == 0 {
            return Ok(Async::Ready(None))
        }

        let mut shared = self.0.borrow_mut();

        let bucket = match shared.queue.pop_front() {
            None => {
                shared.rx_blocked.push_back(task::current());
                return Ok(Async::NotReady);
            }
            Some(mut bucket) => {
                bucket.view_count += 1;
                bucket
            },
        };

        if bucket.view_count < Rc::strong_count(&self.0) {
            let cloned = bucket.data.clone();
            shared.queue.push_front(bucket);
            Ok(Async::Ready(Some(cloned)))
        } else {
            for tx in shared.tx_blocked.drain(..) {
                tx.notify();
            }

            Ok(Async::Ready(Some(bucket.data)))
        }
    }
}
