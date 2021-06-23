use super::DbInterface;

use std::{
    fs::{self, File, OpenOptions},
    io::{self, prelude::*, BufReader, BufWriter},
};

use crate::{
    db::{self, DbMode},
    otp_importer::OTPImporter,
};

pub struct SimpleFileDb {
    keys: Vec<Vec<u8>>,
    imported_keys: Option<Vec<Vec<u8>>>,
    modified: bool,
    file_name: String,
    mode: DbMode,
}

fn open_file(file_name: &str, mode: &DbMode) -> io::Result<File> {
    let mut file_options = OpenOptions::new();
    let result = match mode {
        DbMode::Read => file_options.read(true),
        DbMode::ReadWrite => file_options.read(true).write(true),
    }
    .create(true)
    .open(file_name)?;

    Ok(result)
}

impl SimpleFileDb {
    pub fn new(file_name: &str, mode: DbMode) -> Result<Self, io::Error> {
        let file = open_file(file_name, &mode)?;

        let mut result = Self {
            keys: vec![],
            imported_keys: None,
            modified: false,
            file_name: file_name.to_string(),
            mode,
        };
        result.read_file_contents(&file);

        Ok(result)
    }

    fn read_file_contents(&mut self, file: &File) {
        let mut reader = BufReader::with_capacity(db::BUFFER_SIZE, file);
        let mut buffer = [0; db::DEFAULT_KEY_SIZE];
        self.keys.clear();
        self.modified = false;

        while let Ok(bytes_read) = reader.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            } else if bytes_read != db::DEFAULT_KEY_SIZE {
                continue;
            }

            self.keys.push(buffer.to_vec());
        }
    }

    pub fn imported_keys(&self) -> Option<&Vec<Vec<u8>>> {
        self.imported_keys.as_ref()
    }

    pub fn close(&mut self) {
        if self.mode == DbMode::Read || !self.modified {
            return;
        }

        const TEMP_DB_FILE: &str = "keys_db.tmp";
        if let Err(_) = fs::remove_file(TEMP_DB_FILE) {
            eprintln!("Cannot remove file: {}", TEMP_DB_FILE);
        }

        let file = open_file(TEMP_DB_FILE, &self.mode)
            .expect(&format!("Cannot open file: {}", TEMP_DB_FILE));
        let mut writer = BufWriter::with_capacity(db::BUFFER_SIZE, file);

        if self.imported_keys.is_some() {
            let imported_keys = self.imported_keys.take().unwrap();

            for key in imported_keys {
                let bytes_written = writer
                    .write(&key)
                    .expect(&format!("Couldn't write key: {:?}", key));
                if bytes_written < key.len() {
                    eprintln!(
                        "Problem writing key to DB. {}/{} bytes written.",
                        bytes_written,
                        key.len()
                    );
                    return;
                }
            }
        }

        for key in &self.keys {
            let bytes_written = writer
                .write(&key)
                .expect(&format!("Couldn't write key: {:?}", key));
            if bytes_written < key.len() {
                eprintln!(
                    "Problem writing key to DB. {}/{} bytes written.",
                    bytes_written,
                    key.len()
                );
                return;
            }
        }

        self.modified = false;
        writer.flush().expect(&format!(
            "Error while flushing buffer to file: {}",
            TEMP_DB_FILE
        ));
        drop(writer);

        fs::rename(TEMP_DB_FILE, &self.file_name).expect(&format!(
            "Couldn't rename file from '{}' to '{}'",
            TEMP_DB_FILE, self.file_name,
        ));
    }
}

impl DbInterface for SimpleFileDb {
    /// Adds entropy data from importer to database. Returns number of bytes written or error.
    fn add_entropy(&mut self, importer: &impl OTPImporter) -> Result<usize, io::Error> {
        let entropy_len = importer.entropy_data().len();
        let entries = entropy_len / db::DEFAULT_KEY_SIZE;
        let data = importer.entropy_data();

        for i in 0..entries {
            let start_idx = i * db::DEFAULT_KEY_SIZE;
            let end_idx = start_idx + db::DEFAULT_KEY_SIZE;
            let key = &data[start_idx..end_idx];
            let vec = Vec::from(key);

            if self.imported_keys.is_some() {
                let mut imported_keys = self.imported_keys.take().unwrap();
                imported_keys.push(vec);
                self.imported_keys = Some(imported_keys);
            } else {
                self.imported_keys = Some(vec![vec]);
            }
            self.modified = true;
        }

        Ok(entries * db::DEFAULT_KEY_SIZE)
    }

    fn get_key(&mut self) -> Option<Vec<u8>> {
        let mut result = self.keys.pop();

        if result.is_some() {
            self.modified = true;
        } else if self.imported_keys.is_some() {
            let mut imported_keys = self.imported_keys.take().unwrap();
            result = imported_keys.pop();
            self.keys = imported_keys;
            self.modified = true;
        }

        result
    }

    fn keys(&self) -> Option<&Vec<Vec<u8>>> {
        if self.keys.is_empty() {
            None
        } else {
            Some(&self.keys)
        }
    }
}

impl Drop for SimpleFileDb {
    fn drop(&mut self) {
        self.close();
    }
}
