// 이 파일에는 thread간 통신을 위한 파이프라인이 포함되어야 함.

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use lazy_static::lazy_static;


pub struct ThreadPipeline<T> {
    pub sender: Sender<T>,
    pub receiver: Receiver<T>,
}

impl<T> ThreadPipeline<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        ThreadPipeline { sender, receiver }
    }

    pub fn send(&self, message: T) {
        self.sender.send(message).unwrap();
    }

    pub fn receive(&self) -> T {
        self.receiver.recv().unwrap()
    }
}


lazy_static! {
    
    
}

