use crate::{
    elf::{Offset, Stream},
    utils,
};
use std::error::Error;

pub struct Types {}

impl Types {
    pub fn new(stream: &mut Stream) -> Self {
        match Types::parse_header(stream) {
            Ok(length) => {
                Types::parse_attrs(stream, length);
                Types {}
            }
            Err(e) => {
                utils::warn(&format!("{e}"));
                Types {}
            }
        }
    }

    /// Returns the length of the .debug_info section not counting the header.
    fn parse_header(stream: &mut Stream) -> Result<u64, Box<dyn Error>> {
        // See 7.5.1.1
        let word = stream.read_word()? as usize;
        let mut unit_length = if word == 0xffffffff {
            stream.read_xword()?
        } else {
            word as u64
        };

        let version = stream.read_half()?;
        if version != 4 {
            return Err(format!("bad .debug_info version: {version}").into());
        }
        unit_length -= 2;

        let abrev_offset = stream.read_offset()?; // TODO need to use this
        if stream.reader.sixty_four_bit {
            unit_length -= 8;
        } else {
            unit_length -= 4;
        }

        let _address_size = stream.read_byte()?; // used for segmented addressing
        unit_length -= 1;
        println!("attributes start at 0x{:x}", stream.offset.0);
        println!("abreviations start at 0x{:x}", abrev_offset);

        Ok(unit_length)
    }

    fn parse_attrs(stream: &mut Stream, len: u64) {}
}

// Each debugging information entry begins with an unsigned LEB128 number containing the
// abbreviation code for the entry (this is an index into the abreb table)

// The abbreviation code is followed by a series of attribute values
// Each attribute value is characterized by an attribute name, these are LEB128 numbers
