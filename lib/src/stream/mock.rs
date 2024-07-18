use std::{
    cmp::min,
    io::{self, Read, Write},
    sync::mpsc::{Receiver, Sender},
};

use crate::stream::Stream;

pub struct MockStream {
    pub tx: Sender<Vec<u8>>,
    pub rx: Receiver<Vec<u8>>,
    pub bytes_read: Vec<u8>,
}

impl MockStream {
    pub fn new(tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>) -> Self {
        Self {
            tx,
            rx,
            bytes_read: Vec::new(),
        }
    }
}

impl Stream for MockStream {}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        while let Ok(bytes) = self.rx.try_recv() {
            self.bytes_read.extend_from_slice(&bytes);
        }

        let len: usize = min(buf.len(), self.bytes_read.len());
        buf[..len].copy_from_slice(&self.bytes_read[..len]);
        self.bytes_read = self.bytes_read.split_off(len);
        Ok(len)
    }
}

impl Write for MockStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tx.send(buf.to_vec()).unwrap();

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
