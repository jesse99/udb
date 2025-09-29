//! This module contains support for the debugging support encoded into ELF files. Most
//! of this is in exe files, not core files. Most of this info is encoded into ".debug_FOO"
//! sections, e.g. ".debug_info", ".debug_abbrev", etc. These contain dwarf debug info
//! which are documented here: https://dwarfstd.org/doc/DWARF5.pdf. The readelf source
//! code is also useful and can be found at https://github.com/bminor/binutils-gdb/tree/master/binutils.
use std::error::Error;

pub mod abbrev;
pub mod line;
pub mod symbols;
pub mod types;

pub use abbrev::*;
pub use line::*;
pub use symbols::*;
pub use types::*;

use crate::elf::{Offset, Stream};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)] // figure 20
pub enum AttributeName {
    //                             value & class
    DW_AT_sibling,                 // 0x01 reference
    DW_AT_location,                // 0x02 exprloc, loclistptr
    DW_AT_name,                    // 0x03 string
    DW_AT_ordering,                // 0x09 constant
    DW_AT_byte_size,               // 0x0b constant, exprloc, reference
    DW_AT_bit_offset,              // 0x0c constant, exprloc, reference
    DW_AT_bit_size,                // 0x0d constant, exprloc, reference
    DW_AT_stmt_list,               // 0x10 lineptr
    DW_AT_low_pc,                  // 0x11 address
    DW_AT_high_pc,                 // 0x12 address, constant
    DW_AT_language,                // 0x13 constant
    DW_AT_discr,                   // 0x15 reference
    DW_AT_discr_value,             // 0x16 constant
    DW_AT_visibility,              // 0x17 constant
    DW_AT_import,                  // 0x18 reference
    DW_AT_string_length,           // 0x19 exprloc, loclistptr
    DW_AT_common_reference,        // 0x1a reference
    DW_AT_comp_dir,                // 0x1b string
    DW_AT_const_value,             // 0x1c block, constant, string
    DW_AT_containing_type,         // 0x1d reference
    DW_AT_default_value,           // 0x1e reference
    DW_AT_inline,                  // 0x20 constant
    DW_AT_is_optional,             // 0x21 flag
    DW_AT_lower_bound,             // 0x22 constant, exprloc, reference
    DW_AT_producer,                // 0x25 string
    DW_AT_prototyped,              // 0x27 flag
    DW_AT_return_addr,             // 0x2a exprloc, loclistptr
    DW_AT_start_scope,             // 0x2c Constant, rangelistptr
    DW_AT_bit_stride,              // 0x2e constant, exprloc, reference
    DW_AT_upper_bound,             // 0x2f constant, exprloc, reference
    DW_AT_abstract_origin,         // 0x31 reference
    DW_AT_accessibility,           // 0x32 constant
    DW_AT_address_class,           // 0x33 constant
    DW_AT_artificial,              // 0x34 flag
    DW_AT_base_types,              // 0x35 reference
    DW_AT_calling_convention,      // 0x36 constant
    DW_AT_count,                   // 0x37 constant, exprloc, reference
    DW_AT_data_member_location,    // 0x38 constant, exprloc, loclistptr
    DW_AT_decl_column,             // 0x39 constant
    DW_AT_decl_file,               // 0x3a constant
    DW_AT_decl_line,               // 0x3b constant
    DW_AT_declaration,             // 0x3c flag
    DW_AT_discr_list,              // 0x3d block
    DW_AT_encoding,                // 0x3e constant
    DW_AT_external,                // 0x3f flag
    DW_AT_frame_base,              // 0x40 exprloc, loclistptr
    DW_AT_friend,                  // 0x41 reference
    DW_AT_identifier_case,         // 0x42 constant
    DW_AT_macro_info,              // 0x43 macptr
    DW_AT_namelist_item,           // 0x44 reference
    DW_AT_priority,                // 0x45 reference
    DW_AT_segment,                 // 0x46 exprloc, loclistptr
    DW_AT_specification,           // 0x47 reference
    DW_AT_static_link,             // 0x48 exprloc, loclistptr
    DW_AT_type,                    // 0x49 reference
    DW_AT_use_location,            // 0x4a exprloc, loclistptr
    DW_AT_variable_parameter,      // 0x4b flag
    DW_AT_virtuality,              // 0x4c constant
    DW_AT_vtable_elem_location,    // 0x4d exprloc, loclistptr
    DW_AT_allocated,               // 0x4e constant, exprloc, reference
    DW_AT_associated,              // 0x4f constant, exprloc, reference
    DW_AT_data_location,           // 0x50 exprloc
    DW_AT_byte_stride,             // 0x51 constant, exprloc, reference
    DW_AT_entry_pc,                // 0x52 address
    DW_AT_use_UTF8,                // 0x53 flag
    DW_AT_extension,               // 0x54 reference
    DW_AT_ranges,                  // 0x55 rangelistptr
    DW_AT_trampoline,              // 0x56 address, flag, reference, string
    DW_AT_call_column,             // 0x57 constant
    DW_AT_call_file,               // 0x58 constant
    DW_AT_call_line,               // 0x59 constant
    DW_AT_description,             // 0x5a string
    DW_AT_binary_scale,            // 0x5b constant
    DW_AT_decimal_scale,           // 0x5c constant
    DW_AT_small,                   // 0x5d reference
    DW_AT_decimal_sign,            // 0x5e constant
    DW_AT_digit_count,             // 0x5f constant
    DW_AT_picture_string,          // 0x60 string
    DW_AT_mutable,                 // 0x61 flag
    DW_AT_threads_scaled,          // 0x62 flag
    DW_AT_explicit,                // 0x63 flag
    DW_AT_object_pointer,          // 0x64 reference
    DW_AT_endianity,               // 0x65 constant
    DW_AT_elemental,               // 0x66 flag
    DW_AT_pure,                    // 0x67 flag
    DW_AT_recursive,               // 0x68 flag
    DW_AT_signature,               // ‡ 0x69 reference
    DW_AT_main_subprogram,         // ‡ 0x6a flag
    DW_AT_data_bit_offset,         // ‡ 0x6b constant
    DW_AT_const_expr,              // ‡ 0x6c flag
    DW_AT_enum_class,              // ‡ 0x6d flag
    DW_AT_linkage_name,            // ‡ 0x6e string
    DW_AT_GNU_all_tail_call_sites, // 0x2116 flag, see https://sourceware.org/elfutils/DwarfExtensions
    DW_AT_GNU_all_call_sites,      // 0x2117 flag
    DW_AT_user,                    // [0x2000, 0x3fff) ---
}

#[allow(non_camel_case_types)] // figure 18
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tag {
    //                                  value
    DW_TAG_array_type,               // 0x01
    DW_TAG_class_type,               // 0x02
    DW_TAG_entry_point,              // 0x03
    DW_TAG_enumeration_type,         // 0x04
    DW_TAG_formal_parameter,         // 0x05
    DW_TAG_imported_declaration,     // 0x08
    DW_TAG_label,                    // 0x0a
    DW_TAG_lexical_block,            // 0x0b
    DW_TAG_member,                   // 0x0d
    DW_TAG_pointer_type,             // 0x0f
    DW_TAG_reference_type,           // 0x10
    DW_TAG_compile_unit,             // 0x11
    DW_TAG_string_type,              // 0x12
    DW_TAG_structure_type,           // 0x13
    DW_TAG_subroutine_type,          // 0x15
    DW_TAG_typedef,                  // 0x16
    DW_TAG_union_type,               // 0x17
    DW_TAG_unspecified_parameters,   // 0x18
    DW_TAG_variant,                  // 0x19
    DW_TAG_common_block,             // 0x1a
    DW_TAG_common_inclusion,         // 0x1b
    DW_TAG_inheritance,              // 0x1c
    DW_TAG_inlined_subroutine,       // 0x1d
    DW_TAG_module,                   // 0x1e
    DW_TAG_ptr_to_member_type,       // 0x1f
    DW_TAG_set_type,                 // 0x20
    DW_TAG_subrange_type,            // 0x21
    DW_TAG_with_stmt,                // 0x22
    DW_TAG_access_declaration,       // 0x23
    DW_TAG_base_type,                // 0x24
    DW_TAG_catch_block,              // 0x25
    DW_TAG_const_type,               // 0x26
    DW_TAG_constant,                 // 0x27
    DW_TAG_enumerator,               // 0x28
    DW_TAG_file_type,                // 0x29
    DW_TAG_friend,                   // 0x2a
    DW_TAG_namelist,                 // 0x2b
    DW_TAG_namelist_item,            // 0x2c
    DW_TAG_packed_type,              // 0x2d
    DW_TAG_subprogram,               // 0x2e
    DW_TAG_template_type_parameter,  // 0x2f
    DW_TAG_template_value_parameter, // 0x30
    DW_TAG_thrown_type,              // 0x31
    DW_TAG_try_block,                // 0x32
    DW_TAG_variant_part,             // 0x33
    DW_TAG_variable,                 // 0x34
    DW_TAG_volatile_type,            // 0x35
    DW_TAG_dwarf_procedure,          // 0x36
    DW_TAG_restrict_type,            // 0x37
    DW_TAG_interface_type,           // 0x38
    DW_TAG_namespace,                // 0x39
    DW_TAG_imported_module,          // 0x3a
    DW_TAG_unspecified_type,         // 0x3b
    DW_TAG_partial_unit,             // 0x3c
    DW_TAG_imported_unit,            // 0x3d
    DW_TAG_condition,                // 0x3f
    DW_TAG_shared_type,              // 0x40
    DW_TAG_type_unit,                // ‡, // 0x41
    DW_TAG_rvalue_reference_type,    // ‡, // 0x42
    DW_TAG_template_alias,           // ‡, // 0x43
    DW_TAG_user,                     // [0x4080, 0xffff]
}

#[allow(non_camel_case_types)] // section 7
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormEncoding {
    //                       value & class
    DW_FORM_addr,         // 0x01 address
    DW_FORM_block2,       // 0x03 block
    DW_FORM_block4,       // 0x04 block
    DW_FORM_data2,        // 0x05 constant
    DW_FORM_data4,        // 0x06 constant
    DW_FORM_data8,        // 0x07 constant
    DW_FORM_string,       // 0x08 string
    DW_FORM_block,        // 0x09 block
    DW_FORM_block1,       // 0x0a block
    DW_FORM_data1,        // 0x0b constant
    DW_FORM_flag,         // 0x0c flag
    DW_FORM_sdata,        // 0x0d constant
    DW_FORM_strp,         // 0x0e string
    DW_FORM_udata,        // 0x0f constant
    DW_FORM_ref_addr,     // 0x10 reference
    DW_FORM_ref1,         // 0x11 reference
    DW_FORM_ref2,         // 0x12 reference
    DW_FORM_ref4,         // 0x13 reference
    DW_FORM_ref8,         // 0x14 reference
    DW_FORM_ref_udata,    // 0x15 reference
    DW_FORM_indirect,     // 0x16 (see Section 7.5.3 on page 203)
    DW_FORM_sec_offset, // 0x17 addrptr, lineptr, loclist, loclistsptr, macptr, rnglist, rnglistsptr, stroffsetsptr
    DW_FORM_exprloc,    // 0x18 exprloc
    DW_FORM_flag_present, //0x19 flag
}

impl AttributeName {
    fn from_u64(value: u64) -> Result<Self, Box<dyn Error>> {
        match value {
            0x01 => Ok(AttributeName::DW_AT_sibling),
            0x02 => Ok(AttributeName::DW_AT_location),
            0x03 => Ok(AttributeName::DW_AT_name),
            0x09 => Ok(AttributeName::DW_AT_ordering),
            0x0b => Ok(AttributeName::DW_AT_byte_size),
            0x0c => Ok(AttributeName::DW_AT_bit_offset),
            0x0d => Ok(AttributeName::DW_AT_bit_size),
            0x10 => Ok(AttributeName::DW_AT_stmt_list),
            0x11 => Ok(AttributeName::DW_AT_low_pc),
            0x12 => Ok(AttributeName::DW_AT_high_pc),
            0x13 => Ok(AttributeName::DW_AT_language),
            0x15 => Ok(AttributeName::DW_AT_discr),
            0x16 => Ok(AttributeName::DW_AT_discr_value),
            0x17 => Ok(AttributeName::DW_AT_visibility),
            0x18 => Ok(AttributeName::DW_AT_import),
            0x19 => Ok(AttributeName::DW_AT_string_length),
            0x1a => Ok(AttributeName::DW_AT_common_reference),
            0x1b => Ok(AttributeName::DW_AT_comp_dir),
            0x1c => Ok(AttributeName::DW_AT_const_value),
            0x1d => Ok(AttributeName::DW_AT_containing_type),
            0x1e => Ok(AttributeName::DW_AT_default_value),
            0x20 => Ok(AttributeName::DW_AT_inline),
            0x21 => Ok(AttributeName::DW_AT_is_optional),
            0x22 => Ok(AttributeName::DW_AT_lower_bound),
            0x25 => Ok(AttributeName::DW_AT_producer),
            0x27 => Ok(AttributeName::DW_AT_prototyped),
            0x2a => Ok(AttributeName::DW_AT_return_addr),
            0x2c => Ok(AttributeName::DW_AT_start_scope),
            0x2e => Ok(AttributeName::DW_AT_bit_stride),
            0x2f => Ok(AttributeName::DW_AT_upper_bound),
            0x31 => Ok(AttributeName::DW_AT_abstract_origin),
            0x32 => Ok(AttributeName::DW_AT_accessibility),
            0x33 => Ok(AttributeName::DW_AT_address_class),
            0x34 => Ok(AttributeName::DW_AT_artificial),
            0x35 => Ok(AttributeName::DW_AT_base_types),
            0x36 => Ok(AttributeName::DW_AT_calling_convention),
            0x37 => Ok(AttributeName::DW_AT_count),
            0x38 => Ok(AttributeName::DW_AT_data_member_location),
            0x39 => Ok(AttributeName::DW_AT_decl_column),
            0x3a => Ok(AttributeName::DW_AT_decl_file),
            0x3b => Ok(AttributeName::DW_AT_decl_line),
            0x3c => Ok(AttributeName::DW_AT_declaration),
            0x3d => Ok(AttributeName::DW_AT_discr_list),
            0x3e => Ok(AttributeName::DW_AT_encoding),
            0x3f => Ok(AttributeName::DW_AT_external),
            0x40 => Ok(AttributeName::DW_AT_frame_base),
            0x41 => Ok(AttributeName::DW_AT_friend),
            0x42 => Ok(AttributeName::DW_AT_identifier_case),
            0x43 => Ok(AttributeName::DW_AT_macro_info),
            0x44 => Ok(AttributeName::DW_AT_namelist_item),
            0x45 => Ok(AttributeName::DW_AT_priority),
            0x46 => Ok(AttributeName::DW_AT_segment),
            0x47 => Ok(AttributeName::DW_AT_specification),
            0x48 => Ok(AttributeName::DW_AT_static_link),
            0x49 => Ok(AttributeName::DW_AT_type),
            0x4a => Ok(AttributeName::DW_AT_use_location),
            0x4b => Ok(AttributeName::DW_AT_variable_parameter),
            0x4c => Ok(AttributeName::DW_AT_virtuality),
            0x4d => Ok(AttributeName::DW_AT_vtable_elem_location),
            0x4e => Ok(AttributeName::DW_AT_allocated),
            0x4f => Ok(AttributeName::DW_AT_associated),
            0x50 => Ok(AttributeName::DW_AT_data_location),
            0x51 => Ok(AttributeName::DW_AT_byte_stride),
            0x52 => Ok(AttributeName::DW_AT_entry_pc),
            0x53 => Ok(AttributeName::DW_AT_use_UTF8),
            0x54 => Ok(AttributeName::DW_AT_extension),
            0x55 => Ok(AttributeName::DW_AT_ranges),
            0x56 => Ok(AttributeName::DW_AT_trampoline),
            0x57 => Ok(AttributeName::DW_AT_call_column),
            0x58 => Ok(AttributeName::DW_AT_call_file),
            0x59 => Ok(AttributeName::DW_AT_call_line),
            0x5a => Ok(AttributeName::DW_AT_description),
            0x5b => Ok(AttributeName::DW_AT_binary_scale),
            0x5c => Ok(AttributeName::DW_AT_decimal_scale),
            0x5d => Ok(AttributeName::DW_AT_small),
            0x5e => Ok(AttributeName::DW_AT_decimal_sign),
            0x5f => Ok(AttributeName::DW_AT_digit_count),
            0x60 => Ok(AttributeName::DW_AT_picture_string),
            0x61 => Ok(AttributeName::DW_AT_mutable),
            0x62 => Ok(AttributeName::DW_AT_threads_scaled),
            0x63 => Ok(AttributeName::DW_AT_explicit),
            0x64 => Ok(AttributeName::DW_AT_object_pointer),
            0x65 => Ok(AttributeName::DW_AT_endianity),
            0x66 => Ok(AttributeName::DW_AT_elemental),
            0x67 => Ok(AttributeName::DW_AT_pure),
            0x68 => Ok(AttributeName::DW_AT_recursive),
            0x69 => Ok(AttributeName::DW_AT_signature),
            0x6a => Ok(AttributeName::DW_AT_main_subprogram),
            0x6b => Ok(AttributeName::DW_AT_data_bit_offset),
            0x6c => Ok(AttributeName::DW_AT_const_expr),
            0x6d => Ok(AttributeName::DW_AT_enum_class),
            0x6e => Ok(AttributeName::DW_AT_linkage_name),
            0x2116 => Ok(AttributeName::DW_AT_GNU_all_tail_call_sites),
            0x2117 => Ok(AttributeName::DW_AT_GNU_all_call_sites),
            0x2000..0x3fff => Ok(AttributeName::DW_AT_user),
            _ => Err(format!("unknown attribute name encoding: {value}").into()),
        }
    }
}

impl Tag {
    fn from_u64(value: u64) -> Result<Self, Box<dyn Error>> {
        match value {
            0x01 => Ok(Tag::DW_TAG_array_type),
            0x02 => Ok(Tag::DW_TAG_class_type),
            0x03 => Ok(Tag::DW_TAG_entry_point),
            0x04 => Ok(Tag::DW_TAG_enumeration_type),
            0x05 => Ok(Tag::DW_TAG_formal_parameter),
            0x08 => Ok(Tag::DW_TAG_imported_declaration),
            0x0a => Ok(Tag::DW_TAG_label),
            0x0b => Ok(Tag::DW_TAG_lexical_block),
            0x0d => Ok(Tag::DW_TAG_member),
            0x0f => Ok(Tag::DW_TAG_pointer_type),
            0x10 => Ok(Tag::DW_TAG_reference_type),
            0x11 => Ok(Tag::DW_TAG_compile_unit),
            0x12 => Ok(Tag::DW_TAG_string_type),
            0x13 => Ok(Tag::DW_TAG_structure_type),
            0x15 => Ok(Tag::DW_TAG_subroutine_type),
            0x16 => Ok(Tag::DW_TAG_typedef),
            0x17 => Ok(Tag::DW_TAG_union_type),
            0x18 => Ok(Tag::DW_TAG_unspecified_parameters),
            0x19 => Ok(Tag::DW_TAG_variant),
            0x1a => Ok(Tag::DW_TAG_common_block),
            0x1b => Ok(Tag::DW_TAG_common_inclusion),
            0x1c => Ok(Tag::DW_TAG_inheritance),
            0x1d => Ok(Tag::DW_TAG_inlined_subroutine),
            0x1e => Ok(Tag::DW_TAG_module),
            0x1f => Ok(Tag::DW_TAG_ptr_to_member_type),
            0x20 => Ok(Tag::DW_TAG_set_type),
            0x21 => Ok(Tag::DW_TAG_subrange_type),
            0x22 => Ok(Tag::DW_TAG_with_stmt),
            0x23 => Ok(Tag::DW_TAG_access_declaration),
            0x24 => Ok(Tag::DW_TAG_base_type),
            0x25 => Ok(Tag::DW_TAG_catch_block),
            0x26 => Ok(Tag::DW_TAG_const_type),
            0x27 => Ok(Tag::DW_TAG_constant),
            0x28 => Ok(Tag::DW_TAG_enumerator),
            0x29 => Ok(Tag::DW_TAG_file_type),
            0x2a => Ok(Tag::DW_TAG_friend),
            0x2b => Ok(Tag::DW_TAG_namelist),
            0x2c => Ok(Tag::DW_TAG_namelist_item),
            0x2d => Ok(Tag::DW_TAG_packed_type),
            0x2e => Ok(Tag::DW_TAG_subprogram),
            0x2f => Ok(Tag::DW_TAG_template_type_parameter),
            0x30 => Ok(Tag::DW_TAG_template_value_parameter),
            0x31 => Ok(Tag::DW_TAG_thrown_type),
            0x32 => Ok(Tag::DW_TAG_try_block),
            0x33 => Ok(Tag::DW_TAG_variant_part),
            0x34 => Ok(Tag::DW_TAG_variable),
            0x35 => Ok(Tag::DW_TAG_volatile_type),
            0x36 => Ok(Tag::DW_TAG_dwarf_procedure),
            0x37 => Ok(Tag::DW_TAG_restrict_type),
            0x38 => Ok(Tag::DW_TAG_interface_type),
            0x39 => Ok(Tag::DW_TAG_namespace),
            0x3a => Ok(Tag::DW_TAG_imported_module),
            0x3b => Ok(Tag::DW_TAG_unspecified_type),
            0x3c => Ok(Tag::DW_TAG_partial_unit),
            0x3d => Ok(Tag::DW_TAG_imported_unit),
            0x3f => Ok(Tag::DW_TAG_condition),
            0x40 => Ok(Tag::DW_TAG_shared_type),
            0x41 => Ok(Tag::DW_TAG_type_unit),
            0x42 => Ok(Tag::DW_TAG_rvalue_reference_type),
            0x43 => Ok(Tag::DW_TAG_template_alias),
            0x4080..0xffff => Ok(Tag::DW_TAG_user),
            _ => Err(format!("unknown tag encoding: {value}").into()),
        }
    }
}

impl FormEncoding {
    fn from_u64(value: u64) -> Result<Self, Box<dyn Error>> {
        match value {
            0x01 => Ok(FormEncoding::DW_FORM_addr),
            0x03 => Ok(FormEncoding::DW_FORM_block2),
            0x04 => Ok(FormEncoding::DW_FORM_block4),
            0x05 => Ok(FormEncoding::DW_FORM_data2),
            0x06 => Ok(FormEncoding::DW_FORM_data4),
            0x07 => Ok(FormEncoding::DW_FORM_data8),
            0x08 => Ok(FormEncoding::DW_FORM_string),
            0x09 => Ok(FormEncoding::DW_FORM_block),
            0x0a => Ok(FormEncoding::DW_FORM_block1),
            0x0b => Ok(FormEncoding::DW_FORM_data1),
            0x0c => Ok(FormEncoding::DW_FORM_flag),
            0x0d => Ok(FormEncoding::DW_FORM_sdata),
            0x0e => Ok(FormEncoding::DW_FORM_strp),
            0x0f => Ok(FormEncoding::DW_FORM_udata),
            0x10 => Ok(FormEncoding::DW_FORM_ref_addr),
            0x11 => Ok(FormEncoding::DW_FORM_ref1),
            0x12 => Ok(FormEncoding::DW_FORM_ref2),
            0x13 => Ok(FormEncoding::DW_FORM_ref4),
            0x14 => Ok(FormEncoding::DW_FORM_ref8),
            0x15 => Ok(FormEncoding::DW_FORM_ref_udata),
            0x16 => Ok(FormEncoding::DW_FORM_indirect),
            0x17 => Ok(FormEncoding::DW_FORM_sec_offset),
            0x18 => Ok(FormEncoding::DW_FORM_exprloc),
            0x19 => Ok(FormEncoding::DW_FORM_flag_present),
            _ => Err(format!("unknown form encoding: {value:x}").into()),
        }
    }
}

/// LEB128 encoded
fn decode_u64(stream: &mut Stream) -> Result<u64, Box<dyn Error>> {
    let mut result = 0;
    let mut shift = 0;
    loop {
        let byte = stream.read_byte()? as u64;
        result |= (byte & 0x7F) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
    }
    Ok(result)
}
