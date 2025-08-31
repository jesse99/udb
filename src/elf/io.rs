use crate::repl::HexdumpOffsets;
use crate::utils;
use crate::utils::Styling;
use crate::utils::print_styled;
use memmap2::Mmap;
use std::error::Error;

pub struct Reader {
    pub little_endian: bool,
    pub sixty_four_bit: bool,
    bytes: Mmap,
}

impl Reader {
    /// Note that these functions all return a Result because core files are sometimes
    /// corrupted and we want to continue to work as well as we can when that happens.
    pub fn new(bytes: Mmap) -> Result<Self, Box<dyn Error>> {
        // see https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
        utils::require(bytes.len() > 64, "core file is much too small")?;
        let magic = bytes.get(0..4).unwrap();
        utils::require(
            magic[0] == 0x7f && magic[1] == 0x45 && magic[2] == 0x4c || magic[3] == 0x46,
            "not a core file (bad magic)",
        )?;

        let ei_class = *bytes.get(0x04).unwrap();
        let ei_data = *bytes.get(0x05).unwrap();
        let ei_version = *bytes.get(0x06).unwrap();
        let e_type = *bytes.get(0x10).unwrap();
        utils::require(ei_version == 1, &format!("bad elf version: {ei_version}"))?;
        utils::require(
            e_type == 0x02 || e_type == 0x03 || e_type == 0x04,
            "bad elf type: not a core, exe, or shared lib",
        )?;

        Ok(Reader {
            bytes,
            sixty_four_bit: ei_class == 2,
            little_endian: ei_data == 1,
        })
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn slice(&self, offset: usize, size: usize) -> Result<&[u8], Box<dyn Error>> {
        if offset + size > self.bytes.len() {
            return Err("slice out of bounds".into());
        }
        Ok(&self.bytes[offset..offset + size])
    }

    pub fn read_byte(&self, offset: usize) -> Result<u8, Box<dyn Error>> {
        self.bytes
            .get(offset)
            .ok_or("couldn't read byte at offset".into())
            .copied()
    }

    pub fn read_half(&self, offset: usize) -> Result<u16, Box<dyn Error>> {
        let slice = &self.bytes[offset..offset + 2];
        if self.little_endian {
            Ok(u16::from_le_bytes(slice.try_into()?))
        } else {
            Ok(u16::from_be_bytes(slice.try_into()?))
        }
    }

    pub fn read_word(&self, offset: usize) -> Result<u32, Box<dyn Error>> {
        let slice = &self.bytes[offset..offset + 4];
        if self.little_endian {
            Ok(u32::from_le_bytes(slice.try_into()?))
        } else {
            Ok(u32::from_be_bytes(slice.try_into()?))
        }
    }

    pub fn read_xword(&self, offset: usize) -> Result<u64, Box<dyn Error>> {
        let slice = &self.bytes[offset..offset + 8];
        if self.little_endian {
            Ok(u64::from_le_bytes(slice.try_into()?))
        } else {
            Ok(u64::from_be_bytes(slice.try_into()?))
        }
    }

    /// Read either a u32 or u64 word depending on whether the core file is 64-bit.
    /// But, for sanity, always return the result as 64 bits.
    pub fn read_addr(&self, offset: usize) -> Result<u64, Box<dyn Error>> {
        if self.sixty_four_bit {
            self.read_xword(offset)
        } else {
            Ok(self.read_word(offset)? as u64)
        }
    }

    // TODO should address and offset be new types?
    fn read_offset(&self, offset: usize) -> Result<u64, Box<dyn Error>> {
        if self.sixty_four_bit {
            self.read_xword(offset)
        } else {
            Ok(self.read_word(offset)? as u64)
        }
    }

    pub fn hex_dump(&self, addr: u64, offset: usize, size: usize, offsets: HexdumpOffsets) {
        let mut i = offset;
        loop {
            match offsets {
                HexdumpOffsets::None => (),
                HexdumpOffsets::Addr => {
                    print_styled!("{:012x}: ", hex_offset, addr + (i - offset) as u64);
                }
                HexdumpOffsets::Zero => {
                    print_styled!("{:04x}: ", hex_offset, i - offset);
                }
            }

            for j in 0..8 {
                if i + j >= offset + size || i + j >= self.len() {
                    break;
                }
                print_styled!("{:02x} ", hex_hex, self.read_byte(i + j).unwrap());
            }
            print!(" ");
            for j in 0..8 {
                if i + j >= offset + size || i + j >= self.len() {
                    break;
                }
                print_styled!("{:02x} ", hex_hex, self.read_byte(i + j).unwrap());
            }
            print!("   ");
            for j in 0..16 {
                if i + j >= offset + size || i + j >= self.len() {
                    break;
                }
                let ch = self.read_byte(i + j).unwrap() as char;
                if ch.is_ascii_graphic() {
                    print_styled!("{ch}", hex_ascii);
                } else {
                    print_styled!(".", hex_ascii);
                }
            }
            println!();
            i += 16;
            if i >= offset + size || i >= self.len() {
                break;
            }
        }
    }
}

pub struct Stream<'a> {
    pub reader: &'a Reader,
    pub offset: usize,
}

impl<'a> Stream<'a> {
    pub fn new(reader: &'a Reader, offset: usize) -> Self {
        Stream { reader, offset }
    }

    pub fn read_byte(&mut self) -> Result<u8, Box<dyn Error>> {
        let byte = self.reader.read_byte(self.offset)?;
        self.offset += 1;
        Ok(byte)
    }

    pub fn read_half(&mut self) -> Result<u16, Box<dyn Error>> {
        let half = self.reader.read_half(self.offset)?;
        self.offset += 2;
        Ok(half)
    }

    pub fn read_word(&mut self) -> Result<u32, Box<dyn Error>> {
        let word = self.reader.read_word(self.offset)?;
        self.offset += 4;
        Ok(word)
    }

    pub fn read_xword(&mut self) -> Result<u64, Box<dyn Error>> {
        let xword = self.reader.read_xword(self.offset)?;
        self.offset += 8;
        Ok(xword)
    }

    pub fn read_int(&mut self) -> Result<i32, Box<dyn Error>> {
        let word = self.reader.read_word(self.offset)?;
        self.offset += 4;
        Ok(word as i32)
    }

    /// Corresponds to the kernel's user_long_t which, I think,
    /// is 64 or 32 bits.
    pub fn read_ulong(&mut self) -> Result<u64, Box<dyn Error>> {
        if self.reader.sixty_four_bit {
            let word = self.reader.read_xword(self.offset)?;
            self.offset += 8;
            return Ok(word);
        } else {
            let word = self.reader.read_word(self.offset)?;
            self.offset += 4;
            return Ok(word as u64);
        }
    }

    pub fn read_addr(&mut self) -> Result<u64, Box<dyn Error>> {
        if self.reader.sixty_four_bit {
            let word = self.reader.read_xword(self.offset)?;
            self.offset += 8;
            return Ok(word);
        } else {
            let word = self.reader.read_word(self.offset)?;
            self.offset += 4;
            return Ok(word as u64);
        }
    }

    pub fn read_offset(&mut self) -> Result<u64, Box<dyn Error>> {
        if self.reader.sixty_four_bit {
            let word = self.reader.read_xword(self.offset)?;
            self.offset += 8;
            return Ok(word);
        } else {
            let word = self.reader.read_word(self.offset)?;
            self.offset += 4;
            return Ok(word as u64);
        }
    }

    /// Read a null-terminated ASCII string.
    pub fn read_string(&mut self) -> Result<String, Box<dyn Error>> {
        let mut s = String::new();
        loop {
            // Kernel documents these as ASCII though I'm not sure I believe that.
            let byte = self.read_byte()?;
            if byte == 0 {
                break;
            }
            s.push(byte as char);
        }
        Ok(s)
    }
}
