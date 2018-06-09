extern crate byteorder;
extern crate threadpool;

use std::error::Error;

mod error;
mod handler;
mod message;
mod network;
mod storage;

type Result<T> = std::result::Result<T, Box<Error>>;
