use crate::{
    debug::{AttributeName, FormEncoding, Tag, decode_u64},
    elf::Stream,
};
use std::error::Error;

/// This determines how values are encoded into the .debug_info section.
pub struct Abbreviation {
    /// DW_TAG_compile_unit, DW_TAG_typedef, DW_TAG_base_type, etc
    pub tag: Tag,

    /// If true then subsequent entries are children (until a NULL entry). Otherwise
    /// they are siblings.
    pub has_children: bool,

    /// The type of an attribute in a .debug_info entry along with how the associated
    /// value is encoded.
    pub attrs: Vec<Attribute>,
}

pub struct Attribute {
    pub name: AttributeName,
    pub encoding: FormEncoding,
}

impl Abbreviation {
    pub fn new(stream: &mut Stream) -> Result<Self, Box<dyn Error>> {
        let _code = decode_u64(stream)?; // 1-based index into the abbrev table

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
            tag,
            has_children,
            attrs,
        })
    }
}
