use std::error::Error;

use crate::{
    debug::{AttributeName, FormEncoding, Tag, decode_u64},
    elf::Stream,
};

pub struct Abbreviation {
    /// 1-based index into the abbrev table
    pub code: u64,

    /// DW_TAG_compile_unit, DW_TAG_typedef, DW_TAG_base_type, etc
    pub tag: Tag,
    pub has_children: bool,
    pub attrs: Vec<Attribute>,
}

pub struct Attribute {
    pub name: AttributeName,
    pub encoding: FormEncoding,
}

impl Abbreviation {
    pub fn new(stream: &mut Stream) -> Result<Self, Box<dyn Error>> {
        let code = decode_u64(stream)?; // TODO bail if this is 0

        let tag = decode_u64(stream)?;
        let tag = Tag::from_u64(tag)?;
        let has_children = stream.read_byte()? != 0;

        let mut attrs = Vec::new();
        loop {
            let name = decode_u64(stream)?;
            if name == 0 {
                break;
            }
            let name = AttributeName::from_u64(name)?;

            let encoding = decode_u64(stream)?;
            let encoding = FormEncoding::from_u64(encoding)?;
            attrs.push(Attribute { name, encoding })
        }
        match stream.read_byte() {
            Ok(0) => (),
            Ok(b) => {
                return Err(
                    format!("expected 0 byte to end abbrev entry, but found 0x{b:x}").into(),
                );
            }
            Err(e) => return Err(e),
        }
        Ok(Abbreviation {
            code,
            tag,
            has_children,
            attrs,
        })
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
