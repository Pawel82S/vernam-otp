mod bin_file;
mod text_file_hex;

pub use bin_file::BinaryFileImporter;
pub use text_file_hex::TextFileHexImporter;

pub trait OTPImporter {
    fn entropy_data(&self) -> &Vec<u8>;
}
