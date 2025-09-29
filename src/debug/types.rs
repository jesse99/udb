use crate::{
    debug::{Abbreviations, AttributeEncoding, AttributeName, FormEncoding, Tag, decode_u64},
    elf::{ElfFile, Offset, Stream, StringView},
};
use std::error::Error;

// TODO should we instead construct a high level Type enum?
// or maybe have both this and an enum?
pub struct Type {
    pub tag: Tag,
    pub attrs: Vec<Attribute>,
    pub children: Vec<Type>,
}

#[derive(Debug)]
pub enum TypeLoc {
    /// Offset into the file plus the number of information bytes containing a DWARF expression.
    ExprLoc(Offset, u64),

    /// Offset into the .debug_loc section to the first byte of the data making up the
    /// location list for the compilation unit.
    LocListPtr(u64),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Attribute {
    DW_AT_sibling(u64),
    DW_AT_location(TypeLoc),
    DW_AT_name(StringView),
    // DW_AT_ordering,             // 0x09 constant
    DW_AT_byte_size(u32), // amount of storage needed to hold an instance of the type
    // DW_AT_bit_offset,           // 0x0c constant, exprloc, reference
    // DW_AT_bit_size,             // 0x0d constant, exprloc, reference
    DW_AT_stmt_list(u32), // section offset to the line number information for this compilation unit
    DW_AT_low_pc(u64),    // relocated address of the first instruction associated with the entity
    DW_AT_high_pc(u64),
    DW_AT_language(u16), // TODO use a language enum
    // DW_AT_discr,                // 0x15 reference
    // DW_AT_discr_value,          // 0x16 constant
    // DW_AT_visibility,           // 0x17 constant
    // DW_AT_import,               // 0x18 reference
    // DW_AT_string_length,        // 0x19 exprloc, loclistptr
    // DW_AT_common_reference,     // 0x1a reference
    DW_AT_comp_dir(StringView),
    // DW_AT_const_value,          // 0x1c block, constant, string
    // DW_AT_containing_type,      // 0x1d reference
    // DW_AT_default_value,        // 0x1e reference
    // DW_AT_inline,               // 0x20 constant
    // DW_AT_is_optional,          // 0x21 flag
    // DW_AT_lower_bound,          // 0x22 constant, exprloc, reference
    DW_AT_producer(StringView),
    DW_AT_prototyped(bool),
    // DW_AT_return_addr,          // 0x2a exprloc, loclistptr
    // DW_AT_start_scope,          // 0x2c Constant, rangelistptr
    // DW_AT_bit_stride,           // 0x2e constant, exprloc, reference
    // DW_AT_upper_bound,          // 0x2f constant, exprloc, reference
    // DW_AT_abstract_origin,      // 0x31 reference
    // DW_AT_accessibility,        // 0x32 constant
    // DW_AT_address_class,        // 0x33 constant
    // DW_AT_artificial,           // 0x34 flag
    // DW_AT_base_types,           // 0x35 reference
    // DW_AT_calling_convention,   // 0x36 constant
    // DW_AT_count,                // 0x37 constant, exprloc, reference
    DW_AT_data_member_location(TypeLoc),
    DW_AT_decl_column(u32),
    DW_AT_decl_file(u32),
    DW_AT_decl_line(u32),
    DW_AT_declaration(bool),
    // DW_AT_discr_list,           // 0x3d block
    DW_AT_encoding(u8), // TODO use an enum
    DW_AT_external(bool),
    DW_AT_frame_base(TypeLoc),
    // DW_AT_friend,               // 0x41 reference
    // DW_AT_identifier_case,      // 0x42 constant
    // DW_AT_macro_info,           // 0x43 macptr
    // DW_AT_namelist_item,        // 0x44 reference
    // DW_AT_priority,             // 0x45 reference
    // DW_AT_segment,              // 0x46 exprloc, loclistptr
    // DW_AT_specification,        // 0x47 reference
    // DW_AT_static_link,          // 0x48 exprloc, loclistptr
    DW_AT_type(u64), // offset from the first byte of the compilation header for the compilation unit containing the reference
    // DW_AT_use_location,         // 0x4a exprloc, loclistptr
    // DW_AT_variable_parameter,   // 0x4b flag
    // DW_AT_virtuality,           // 0x4c constant
    // DW_AT_vtable_elem_location, // 0x4d exprloc, loclistptr
    // DW_AT_allocated,            // 0x4e constant, exprloc, reference
    // DW_AT_associated,           // 0x4f constant, exprloc, reference
    // DW_AT_data_location,        // 0x50 exprloc
    // DW_AT_byte_stride,          // 0x51 constant, exprloc, reference
    // DW_AT_entry_pc,             // 0x52 address
    // DW_AT_use_UTF8,             // 0x53 flag
    // DW_AT_extension,            // 0x54 reference
    // DW_AT_ranges,               // 0x55 rangelistptr
    // DW_AT_trampoline,           // 0x56 address, flag, reference, string
    // DW_AT_call_column,          // 0x57 constant
    // DW_AT_call_file,            // 0x58 constant
    // DW_AT_call_line,            // 0x59 constant
    // DW_AT_description,          // 0x5a string
    // DW_AT_binary_scale,         // 0x5b constant
    // DW_AT_decimal_scale,        // 0x5c constant
    // DW_AT_small,                // 0x5d reference
    // DW_AT_decimal_sign,         // 0x5e constant
    // DW_AT_digit_count,          // 0x5f constant
    // DW_AT_picture_string,       // 0x60 string
    // DW_AT_mutable,              // 0x61 flag
    // DW_AT_threads_scaled,       // 0x62 flag
    // DW_AT_explicit,             // 0x63 flag
    // DW_AT_object_pointer,       // 0x64 reference
    // DW_AT_endianity,            // 0x65 constant
    // DW_AT_elemental,            // 0x66 flag
    // DW_AT_pure,                 // 0x67 flag
    // DW_AT_recursive,            // 0x68 flag
    // DW_AT_signature,            // ‡ 0x69 reference
    // DW_AT_main_subprogram,      // ‡ 0x6a flag
    // DW_AT_data_bit_offset,      // ‡ 0x6b constant
    // DW_AT_const_expr,           // ‡ 0x6c flag
    // DW_AT_enum_class,           // ‡ 0x6d flag
    // DW_AT_linkage_name,         // ‡ 0x6e string
    DW_AT_GNU_all_tail_call_sites(bool), // 0x2116 flag, see https://sourceware.org/elfutils/DwarfExtensions
    DW_AT_GNU_all_call_sites(bool),      // 0x2117 flag
                                         // DW_AT_user,                 // [0x2000, 0x3fff) ---
}

pub struct ParseTypes<'a> {
    exe: &'a ElfFile,
    values: Offset,          // offset to .debug_info + header
    end: Offset,             // .debug_info end
    strings: Option<Offset>, // offset to .debug_str
    addr_size: u8,
    abbrevs: Vec<Abbreviations>,
    sixty_four: bool,
}

impl<'a> ParseTypes<'a> {
    pub fn new(exe: &'a ElfFile) -> Result<Self, Box<dyn Error>> {
        if let Some(section) = exe.find_section_named(".debug_info") {
            let mut stream = Stream::new(exe.reader, section.obytes.start);
            let abbrevs = exe.find_abbreviations();
            let strings = exe.find_section_named(".debug_str").map(|s| s.obytes.start);
            match ParseTypes::parse_header(&mut stream) {
                Ok((sixty_four, length, addr_size)) => Ok(ParseTypes {
                    exe,
                    abbrevs,
                    values: stream.offset,
                    end: stream.offset + length as i64,
                    strings,
                    addr_size,
                    sixty_four,
                }),
                Err(e) => Err(e),
            }
        } else {
            Err("couldn't find section .debug_info".into())
        }
    }

    pub fn parse(&self) -> Vec<Type> {
        let mut stream = Stream::new(self.exe.reader, self.values);
        match self.parse_types(&mut stream) {
            (t, None) => t,
            (t, Some(e)) => {
                println!("error parsing .debug_info types: {e}");
                t
            }
        }
    }

    // Returns as many types as possible along with an indication of whether there was
    // an error.
    fn parse_types(&self, stream: &mut Stream) -> (Vec<Type>, Option<Box<dyn Error>>) {
        let mut types = Vec::new();
        loop {
            match self.parse_type(stream) {
                (None, None) => return (types, None),
                (None, Some(err)) => return (types, Some(err)),
                (Some(t), None) => types.push(t),
                (Some(t), Some(e)) => {
                    types.push(t);
                    return (types, Some(e));
                }
            }
            if stream.offset >= self.end {
                // let err = Box::<dyn Error>::from("parse_types over-read");
                // return (types, Some(err));
                return (types, None);
            }
        }
    }

    /// Returns the length of the .debug_info section not counting the header.
    fn parse_header(stream: &mut Stream) -> Result<(bool, u64, u8), Box<dyn Error>> {
        // See 7.5.1.1
        let word = stream.read_word()? as usize;
        let (sixty_four, mut values_length) = if word == 0xffffffff {
            (true, stream.read_xword()?)
        } else {
            (false, word as u64)
        };

        let version = stream.read_half()?;
        if version != 2 && version != 4 {
            // docs say 4 but seeing 2
            return Err(format!("bad .debug_info version: {version}").into());
        }
        values_length -= 2;

        let abrev_offset = if sixty_four {
            // TODO need to use this
            stream.read_offset()?
        } else {
            stream.read_word()? as u64
        };
        if sixty_four {
            values_length -= 8;
        } else {
            values_length -= 4;
        }

        let address_size = stream.read_byte()?; // used for segmented addressing
        values_length -= 1;
        println!("values start at 0x{:x}", stream.offset.0);
        println!("abreviations start at 0x{:x}", abrev_offset);
        println!("address_size: {address_size}");
        println!("values_length: {values_length}");

        Ok((sixty_four, values_length, address_size))
    }

    fn parse_type(&self, stream: &mut Stream) -> (Option<Type>, Option<Box<dyn Error>>) {
        let code = match decode_u64(stream) {
            Ok(0) => return (None, None),
            Ok(c) => c as usize,
            Err(e) => return (None, Some(e)),
        };
        if code > self.abbrevs.len() {
            let err = Box::<dyn Error>::from("attr code is too large");
            return (None, Some(err));
        }
        // println!("code: {} tag: {:?}", code, self.abbrevs[code - 1].tag);
        let attrs = match self.parse_attrs(stream, code - 1) {
            Ok(a) => a,
            Err(e) => return (None, Some(e)),
        };
        let children = if self.abbrevs[code - 1].has_children {
            match self.parse_types(stream) {
                (t, None) => t,
                (t, e) => {
                    return (
                        Some(Type {
                            tag: self.abbrevs[code - 1].tag,
                            attrs,
                            children: t,
                        }),
                        e,
                    );
                }
            }
        } else {
            vec![]
        };
        (
            Some(Type {
                tag: self.abbrevs[code - 1].tag,
                attrs,
                children,
            }),
            None,
        )
    }

    fn parse_attrs(
        &self,
        stream: &mut Stream,
        abbrev_index: usize,
    ) -> Result<Vec<Attribute>, Box<dyn Error>> {
        let abbrev = &self.abbrevs[abbrev_index];
        let mut attrs = Vec::with_capacity(abbrev.attrs.len());
        for ae in abbrev.attrs.iter() {
            let attr = self.parse_attr(stream, ae)?;
            attrs.push(attr);
        }
        Ok(attrs)
    }

    fn parse_attr(
        &self,
        stream: &mut Stream,
        ae: &AttributeEncoding,
    ) -> Result<Attribute, Box<dyn Error>> {
        let a = match ae.name {
            AttributeName::DW_AT_sibling => {
                Attribute::DW_AT_sibling(self.parse_ref(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_location => {
                Attribute::DW_AT_location(self.parse_exprloc(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_name => {
                Attribute::DW_AT_name(self.parse_str(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_ordering => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_byte_size => {
                Attribute::DW_AT_byte_size(self.parse_u32(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_bit_offset => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_bit_size => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_stmt_list => {
                Attribute::DW_AT_stmt_list(self.parse_u32(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_low_pc => Attribute::DW_AT_low_pc(self.parse_addr(stream)?),
            AttributeName::DW_AT_high_pc => Attribute::DW_AT_high_pc(self.parse_addr(stream)?), // TODO can be a constant (which is added to low_pc)
            AttributeName::DW_AT_language => {
                Attribute::DW_AT_language(self.parse_u16(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_discr => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_discr_value => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_visibility => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_import => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_string_length => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_common_reference => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_comp_dir => {
                Attribute::DW_AT_comp_dir(self.parse_str(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_const_value => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_containing_type => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_default_value => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_inline => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_is_optional => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_lower_bound => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_producer => {
                Attribute::DW_AT_producer(self.parse_str(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_prototyped => {
                Attribute::DW_AT_prototyped(self.parse_flag(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_return_addr => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_start_scope => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_bit_stride => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_upper_bound => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_abstract_origin => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_accessibility => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_address_class => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_artificial => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_base_types => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_calling_convention => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_count => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_data_member_location => {
                Attribute::DW_AT_data_member_location(self.parse_exprloc(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_decl_column => {
                Attribute::DW_AT_decl_column(self.parse_u32(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_decl_file => {
                Attribute::DW_AT_decl_file(self.parse_u32(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_decl_line => {
                Attribute::DW_AT_decl_line(self.parse_u32(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_declaration => {
                Attribute::DW_AT_declaration(self.parse_flag(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_discr_list => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_encoding => {
                Attribute::DW_AT_encoding(self.parse_u8(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_external => {
                Attribute::DW_AT_external(self.parse_flag(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_frame_base => {
                Attribute::DW_AT_frame_base(self.parse_exprloc(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_friend => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_identifier_case => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_macro_info => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_namelist_item => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_priority => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_segment => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_specification => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_static_link => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_type => {
                Attribute::DW_AT_type(self.parse_ref(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_use_location => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_variable_parameter => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_virtuality => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_vtable_elem_location => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_allocated => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_associated => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_data_location => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_byte_stride => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_entry_pc => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_use_UTF8 => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_extension => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_ranges => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_trampoline => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_call_column => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_call_file => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_call_line => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_description => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_binary_scale => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_decimal_scale => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_small => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_decimal_sign => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_digit_count => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_picture_string => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_mutable => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_threads_scaled => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_explicit => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_object_pointer => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_endianity => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_elemental => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_pure => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_recursive => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_signature => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_main_subprogram => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_data_bit_offset => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_const_expr => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_enum_class => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_linkage_name => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_user => {
                return Err(format!(
                    "{:?} not implemented for encoding {:?}",
                    ae.name, ae.encoding
                )
                .into());
            }
            AttributeName::DW_AT_GNU_all_tail_call_sites => {
                // TODO there are more of these
                Attribute::DW_AT_GNU_all_tail_call_sites(self.parse_flag(stream, ae.encoding)?)
            }
            AttributeName::DW_AT_GNU_all_call_sites => {
                Attribute::DW_AT_GNU_all_call_sites(self.parse_flag(stream, ae.encoding)?)
            }
        };
        Ok(a)
    }

    fn parse_u8(&self, stream: &mut Stream, encoding: FormEncoding) -> Result<u8, Box<dyn Error>> {
        match encoding {
            FormEncoding::DW_FORM_data1 => self.parse_data1(stream),
            _ => Err(format!("parse_u8 didn't expect {encoding:?}").into()),
        }
    }

    fn parse_u16(
        &self,
        stream: &mut Stream,
        encoding: FormEncoding,
    ) -> Result<u16, Box<dyn Error>> {
        match encoding {
            FormEncoding::DW_FORM_data1 => Ok(self.parse_data1(stream)? as u16),
            FormEncoding::DW_FORM_data2 => self.parse_data2(stream),
            FormEncoding::DW_FORM_sdata => todo!(),
            FormEncoding::DW_FORM_udata => todo!(),
            _ => Err(format!("parse_u16 didn't expect {encoding:?}").into()),
        }
    }

    fn parse_u32(
        &self,
        stream: &mut Stream,
        encoding: FormEncoding,
    ) -> Result<u32, Box<dyn Error>> {
        match encoding {
            FormEncoding::DW_FORM_data1 => Ok(self.parse_data1(stream)? as u32),
            FormEncoding::DW_FORM_data2 => Ok(self.parse_data2(stream)? as u32),
            FormEncoding::DW_FORM_data4 => self.parse_data4(stream),
            FormEncoding::DW_FORM_sdata => todo!(),
            FormEncoding::DW_FORM_udata => todo!(),
            _ => Err(format!("parse_u32 didn't expect {encoding:?}").into()),
        }
    }

    // fn parse_u64(
    //     &self,
    //     stream: &mut Stream,
    //     encoding: FormEncoding,
    // ) -> Result<u64, Box<dyn Error>> {
    //     match encoding {
    //         FormEncoding::DW_FORM_data1 => Ok(self.parse_data1(stream)? as u64),
    //         FormEncoding::DW_FORM_data2 => Ok(self.parse_data2(stream)? as u64),
    //         FormEncoding::DW_FORM_data4 => Ok(self.parse_data4(stream)? as u64),
    //         FormEncoding::DW_FORM_data8 => self.parse_data8(stream),
    //         FormEncoding::DW_FORM_sdata => todo!(),
    //         FormEncoding::DW_FORM_udata => todo!(),
    //         _ => Err(format!("parse_u64 didn't expect {encoding:?}").into()),
    //     }
    // }

    fn parse_exprloc(
        // TODO can also be const
        &self,
        stream: &mut Stream,
        encoding: FormEncoding,
    ) -> Result<TypeLoc, Box<dyn Error>> {
        fn exprloc(stream: &mut Stream, encoding: FormEncoding) -> Result<TypeLoc, Box<dyn Error>> {
            let length = match encoding {
                FormEncoding::DW_FORM_block1 => stream.read_byte()? as u64,
                FormEncoding::DW_FORM_block2 => stream.read_half()? as u64,
                FormEncoding::DW_FORM_block4 => stream.read_word()? as u64,
                FormEncoding::DW_FORM_block => decode_u64(stream)?,
                _ => return Err(format!("exprloc didn't expect {encoding:?}").into()),
            };
            let offset = stream.offset;
            stream.offset = stream.offset + length as i64;
            Ok(TypeLoc::ExprLoc(offset, length))
        }

        fn loclistptr(
            stream: &mut Stream,
            encoding: FormEncoding,
        ) -> Result<TypeLoc, Box<dyn Error>> {
            let offset = match encoding {
                FormEncoding::DW_FORM_data1 => stream.read_byte()? as u64,
                FormEncoding::DW_FORM_data2 => stream.read_half()? as u64,
                FormEncoding::DW_FORM_data4 => stream.read_word()? as u64,
                FormEncoding::DW_FORM_data8 => decode_u64(stream)?,
                _ => return Err(format!("loclistptr didn't expect {encoding:?}").into()),
            };
            Ok(TypeLoc::LocListPtr(offset))
        }

        match encoding {
            FormEncoding::DW_FORM_block1
            | FormEncoding::DW_FORM_block2
            | FormEncoding::DW_FORM_block4
            | FormEncoding::DW_FORM_block => exprloc(stream, encoding),
            FormEncoding::DW_FORM_data1
            | FormEncoding::DW_FORM_data2
            | FormEncoding::DW_FORM_data4
            | FormEncoding::DW_FORM_data8 => loclistptr(stream, encoding),
            FormEncoding::DW_FORM_exprloc => todo!(),
            _ => return Err(format!("parse_exprloc didn't expect {encoding:?}").into()),
        }
    }

    // fn parse_block(
    //     &self,
    //     stream: &mut Stream,
    //     encoding: FormEncoding,
    // ) -> Result<(Offset, u64), Box<dyn Error>> {
    //     let length = match encoding {
    //         FormEncoding::DW_FORM_block1 => stream.read_byte()? as u64,
    //         FormEncoding::DW_FORM_block2 => stream.read_half()? as u64,
    //         FormEncoding::DW_FORM_block4 => stream.read_word()? as u64,
    //         FormEncoding::DW_FORM_block => decode_u64(stream)?,
    //         _ => return Err(format!("parse_block didn't expect {encoding:?}").into()),
    //     };
    //     let offset = stream.offset;
    //     stream.offset = stream.offset + length as i64;
    //     Ok((offset, length))
    // }

    fn parse_flag(
        &self,
        stream: &mut Stream,
        encoding: FormEncoding,
    ) -> Result<bool, Box<dyn Error>> {
        match encoding {
            FormEncoding::DW_FORM_flag => Ok(stream.read_byte()? != 0),
            FormEncoding::DW_FORM_flag_present => Ok(true),
            _ => Err(format!("parse_flag didn't expect {encoding:?}").into()),
        }
    }

    fn parse_str(
        &self,
        stream: &mut Stream,
        encoding: FormEncoding,
    ) -> Result<StringView, Box<dyn Error>> {
        match encoding {
            FormEncoding::DW_FORM_string => self.parse_string(stream),
            FormEncoding::DW_FORM_strp => self.parse_strp(stream),
            _ => Err(format!("parse_str didn't expect {encoding:?}").into()),
        }
    }

    fn parse_ref(
        &self,
        stream: &mut Stream,
        encoding: FormEncoding,
    ) -> Result<u64, Box<dyn Error>> {
        match encoding {
            FormEncoding::DW_FORM_ref_addr => todo!(),
            FormEncoding::DW_FORM_ref1 => Ok(self.parse_data1(stream)? as u64),
            FormEncoding::DW_FORM_ref2 => Ok(self.parse_data2(stream)? as u64),
            FormEncoding::DW_FORM_ref4 => Ok(self.parse_data4(stream)? as u64),
            FormEncoding::DW_FORM_ref8 => self.parse_data8(stream),
            FormEncoding::DW_FORM_ref_udata => todo!(),
            _ => Err(format!("parse_ref didn't expect {encoding:?}").into()),
        }
    }

    // See section 7.5.4 for encoding details

    // DW_FORM_addr
    fn parse_addr(&self, stream: &mut Stream) -> Result<u64, Box<dyn Error>> {
        if self.addr_size == 4 {
            Ok(stream.read_word()? as u64)
        } else if self.addr_size == 8 {
            stream.read_xword()
        } else {
            Err(format!("bad addr size: {}", self.addr_size).into())
        }
    }

    // DW_FORM_data1 or DW_FORM_ref1
    fn parse_data1(&self, stream: &mut Stream) -> Result<u8, Box<dyn Error>> {
        stream.read_byte()
    }

    // DW_FORM_data2 or DW_FORM_ref2
    fn parse_data2(&self, stream: &mut Stream) -> Result<u16, Box<dyn Error>> {
        stream.read_half()
    }

    // DW_FORM_data4 or DW_FORM_ref4
    fn parse_data4(&self, stream: &mut Stream) -> Result<u32, Box<dyn Error>> {
        stream.read_word()
    }

    // DW_FORM_data8
    fn parse_data8(&self, stream: &mut Stream) -> Result<u64, Box<dyn Error>> {
        stream.read_xword()
    }

    // DW_FORM_strp
    fn parse_strp(&self, stream: &mut Stream) -> Result<StringView, Box<dyn Error>> {
        let delta = if self.sixty_four {
            // into the .debug_str section
            stream.read_xword()? as i64
        } else {
            stream.read_word()? as i64
        };
        if let Some(start) = self.strings {
            Ok(StringView::new(stream.reader, start + delta))
        } else {
            Err("no .debug_str section".into())
        }
    }

    // DW_FORM_string
    fn parse_string(&self, stream: &mut Stream) -> Result<StringView, Box<dyn Error>> {
        let result = StringView::new(stream.reader, stream.offset);
        loop {
            let byte = stream.read_byte()?;
            if byte == 0 {
                break;
            }
        }
        Ok(result)
    }
}
