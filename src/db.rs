mod simple_file;

pub use simple_file::SimpleFileDb;

use std::io;

use crate::otp_importer::OTPImporter;

// By averge sentences have around 16 words. Averge word lenght is 5 characters. 16 * 5 = 80
pub const DEFAULT_KEY_SIZE: usize = 96;
pub const BUFFER_KEY_CAPACITY: usize = 1000;
pub const BUFFER_SIZE: usize = DEFAULT_KEY_SIZE * BUFFER_KEY_CAPACITY;

#[derive(PartialEq)]
pub enum DbMode {
    Read,
    ReadWrite,
}

pub trait DbInterface {
    fn add_entropy(&mut self, importer: &impl OTPImporter) -> Result<usize, io::Error>;
    fn keys(&self) -> Option<&Vec<Vec<u8>>>;
    fn get_key(&mut self) -> Option<Vec<u8>>;
}
