pub mod mock;
pub mod mock_handler;

use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub trait Stream: Read + Write + Send {}

impl Stream for TcpStream {}
