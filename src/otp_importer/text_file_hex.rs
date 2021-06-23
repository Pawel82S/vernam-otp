use super::OTPImporter;
use std::{fs, io, path::Path};

pub struct TextFileHexImporter {
    data: Vec<u8>,
}

impl TextFileHexImporter {
    pub fn new(file: &Path) -> Result<Self, io::Error> {
        let contents = fs::read_to_string(file)?;

        let data = parse_text(&contents);

        Ok(Self { data })
    }
}

fn parse_text(text: &str) -> Vec<u8> {
    let mut result = vec![];

    for line in text.lines() {
        for hex in line.trim().split(' ') {
            result.push(hex_text_to_u8(hex));
        }
    }

    result
}

fn hex_text_to_u8(hex_text: &str) -> u8 {
    let hex_text_len = hex_text.len();

    if hex_text_len > 2 || hex_text_len == 0 {
        panic!(
            "Not enough or too much characters to parse: {} (correct is 1 or 2)",
            hex_text_len
        );
    } else {
        let hex_char_to_u8 = |ch| match ch {
            'F' => 15,
            'E' => 14,
            'D' => 13,
            'C' => 12,
            'B' => 11,
            'A' => 10,
            '9' => 9,
            '8' => 8,
            '7' => 7,
            '6' => 6,
            '5' => 5,
            '4' => 4,
            '3' => 3,
            '2' => 2,
            '1' => 1,
            '0' => 0,
            _ => panic!("Invalid hexadecimal character: {}", ch),
        };

        hex_text
            .to_uppercase()
            .chars()
            // I have to reverse order of characters in hex string so last will get lowest index in
            // enumerate because this index is used as a power for base 16 from LSB to MSB.
            .rev()
            .enumerate()
            .map(|(idx, ch)| hex_char_to_u8(ch) * 16u8.pow(idx as u32))
            .sum()
    }
}

impl OTPImporter for TextFileHexImporter {
    fn entropy_data(&self) -> &Vec<u8> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_hex_number_parsing() {
        assert_eq!(hex_text_to_u8("00"), 00);
        assert_eq!(hex_text_to_u8("FF"), 255);
        assert_eq!(hex_text_to_u8("0A"), 10);
    }

    #[test]
    #[should_panic]
    fn invalid_hex_number_parsing() {
        hex_text_to_u8("GG");
    }

    #[test]
    fn text_parsing() {
        let text = r" 01 0A FF 00 ";
        assert_eq!(parse_text(text), vec![1, 10, 255, 0]);
    }
}
