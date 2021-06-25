const DEFAULT_DB_FILE: &str = "keys.db";
const MAX_PASSWORD_LENGTH: usize = 30;

mod config;
mod db;
mod otp_importer;

use std::{
    fs::File,
    io::{self, stdin, BufReader, BufWriter, Read, Write},
};

use config::Config;
use db::{DbInterface, DbMode, SimpleFileDb};
use otp_importer::{BinaryFileImporter, TextFileHexImporter};
use structopt::StructOpt;

fn main() {
    let config = Config::from_args();
    // NOTE: Leave it here if some debug information are needed.
    //println!("Config {:#?}", config);

    let mut db = SimpleFileDb::new(DEFAULT_DB_FILE, DbMode::ReadWrite).unwrap();

    if config.statistics() {
        let key_count = if db.keys().is_some() {
            db.keys().unwrap().len()
        } else {
            0
        };
        print_statistics(key_count, db::DEFAULT_KEY_SIZE);
    }

    if config.import_otp() {
        if config.input_file().is_some() && config.input_file().unwrap().exists() {
            if config.import_binary() {
                binary_file_import(&config, &mut db)
            } else {
                text_file_hex_import(&config, &mut db)
            }
            .unwrap_or_else(|e| eprintln!("Couln't read data from file: {}", e));
        } else {
            eprintln!("File to import data from  doesn't exist!");
        }

        // If I import entropy to database no further actions are required. Just end program.
        return;
    }

    let mut data = vec![];
    // Make sure we have input file name to even consider encryption/decryption
    if config.input_file().is_some() {
        // Ask user for password
        let password = get_password(MAX_PASSWORD_LENGTH);
        data = read_input_file(&config, &mut db, &password);

        if config.statistics() {
            let db_keys = if db.keys().is_some() {
                db.keys().unwrap().len()
            } else {
                0
            };
            let imported_keys = if db.imported_keys().is_some() {
                db.imported_keys().unwrap().len()
            } else {
                0
            };

            let total_keys = db_keys + imported_keys;
            print_statistics(total_keys, db::DEFAULT_KEY_SIZE);
        }
    }

    if config.output_file().is_some() {
        write_output_file(&config, &data);
    } else {
        for d in data {
            println!("{:?}", d)
        }
    }
}

fn print_statistics(key_count: usize, key_size: usize) {
    println!("You have {} keys in database.", key_count);
    println!("Each key can encrypt {} bytes of data", key_size);

    let bytes = key_count * key_size;
    println!(
        "You can encrypt/decrypt {} bytes in total ({:.2} KiB).",
        bytes,
        bytes as f64 / 1024.0
    );
}

fn binary_file_import(config: &Config, db: &mut impl DbInterface) -> io::Result<()> {
    let import_file = BinaryFileImporter::new(config.input_file().unwrap())?;
    db.add_entropy(&import_file).unwrap();
    Ok(())
}

fn text_file_hex_import(config: &Config, db: &mut impl DbInterface) -> io::Result<()> {
    let import_file = TextFileHexImporter::new(config.input_file().unwrap())?;
    db.add_entropy(&import_file).unwrap();
    Ok(())
}

fn get_password(max_length: usize) -> Vec<u8> {
    println!(
        "Enter password for encryption/decryption. Max length is {} bytes.",
        max_length
    );
    let mut buffer = String::with_capacity(max_length * 2);
    stdin().read_line(&mut buffer).unwrap();

    let bytes_buffer = buffer.into_bytes();

    // Take first max_length bytes from password. Ignore the rest.
    bytes_buffer[..max_length.min(bytes_buffer.len())].to_vec()
}

fn read_input_file(config: &Config, db: &mut impl DbInterface, password: &[u8]) -> Vec<Vec<u8>> {
    let input_file_name = config.input_file().unwrap();
    let input_file = File::open(input_file_name).unwrap();
    let mut input_buff_reader = BufReader::new(&input_file);
    let mut buffer = [0; db::DEFAULT_KEY_SIZE];

    let mut result = vec![];
    while let Ok(bytes_read) = input_buff_reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        if let Some(entropy_key) = db.get_key() {
            result.push(encrypt(&buffer, &entropy_key, &password));
        } else {
            eprintln!("Not enough keys in database to encrypt/decrypt file.");
            break;
        }
    }

    result
}

fn write_output_file(config: &Config, data: &Vec<Vec<u8>>) {
    let output_file_name = config.output_file().unwrap();
    let output_file = File::create(output_file_name).unwrap();
    let mut output_buff_writer = BufWriter::new(&output_file);

    for d in data {
        output_buff_writer.write(d).unwrap();
    }
}

fn encrypt(data: &[u8], entropy: &[u8], password: &[u8]) -> Vec<u8> {
    if entropy.len() < data.len() {
        panic!("Not enough entropy data {}/{}.", entropy.len(), data.len());
    }

    let mut result = vec![];

    for idx in 0..entropy.len() {
        let mut byte = entropy[idx];

        if idx < data.len() {
            byte ^= data[idx]
        }

        if !password.is_empty() {
            byte ^= password[idx % password.len()]
        }

        result.push(byte);
    }

    result
}
