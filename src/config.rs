use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "vernam-otp", about = "Encryption/decryption program using OTP")]
pub struct Config {
    /// Import OTP file to database
    #[structopt(short = "I", long = "import")]
    import_otp: bool,

    /// File format is binary not hex text file
    #[structopt(short = "b", long = "binary")]
    import_binary: bool,

    /// Show database statistics
    #[structopt(short = "S", long = "database-statistics")]
    statistics: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input_file: Option<PathBuf>,

    /// Output file (print to std if not set)
    #[structopt(parse(from_os_str))]
    output_file: Option<PathBuf>,
}

impl Config {
    pub fn import_otp(&self) -> bool {
        self.import_otp
    }

    pub fn import_binary(&self) -> bool {
        self.import_binary
    }

    pub fn statistics(&self) -> bool {
        self.statistics
    }

    pub fn input_file(&self) -> Option<&PathBuf> {
        self.input_file.as_ref()
    }

    pub fn output_file(&self) -> Option<&PathBuf> {
        self.output_file.as_ref()
    }
}
