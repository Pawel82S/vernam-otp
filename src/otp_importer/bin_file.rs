use super::OTPImporter;
use crate::db;
use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

pub struct BinaryFileImporter {
    data: Vec<u8>,
}

impl BinaryFileImporter {
    pub fn new(file: &Path) -> Result<Self, io::Error> {
        let file = File::open(file).unwrap();
        let mut file_reader = BufReader::with_capacity(db::BUFFER_SIZE, &file);
        let mut data = vec![];
        let mut buffer = [0; db::DEFAULT_KEY_SIZE];

        while let Ok(bytes_read) = file_reader.read(&mut buffer) {
            if bytes_read != db::DEFAULT_KEY_SIZE {
                break;
            }

            data.append(buffer.to_vec().as_mut());
        }

        Ok(Self { data })
    }
}

impl OTPImporter for BinaryFileImporter {
    fn entropy_data(&self) -> &Vec<u8> {
        &self.data
    }
}
