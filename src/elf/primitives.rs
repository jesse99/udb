use crate::elf::Reader;
use std::fmt;
use std::ops::{Add, AddAssign, Sub};

/// Index into the section table.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SectionIndex(pub u32);

/// Index into a string table.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StringIndex(pub u32);

/// An index into a byte within an ELF file.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Offset(pub u64);

/// The address of a byte in the process that cored. These are normally associated with
/// one of the load segments in the core file.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VirtualAddr(pub u64);

/// An address in an exe file. These will be relative to a memory mapped segment in the
/// core file.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct RelativeAddr(pub u64); // TODO can we make this 32 bits?

/// A range of bytes that can be addressed using either offsets into an ELF file or
/// virtual addresses. In general bytes can always be addressed using offsets and bytes
/// within load segments also be addressed using virtual addresses.
#[derive(Copy, Clone)]
pub struct Bytes<A>
where
    A: Add<i64, Output = A> + Copy + Ord,
{
    pub start: A,
    pub size: usize,
}

/// Points to a null-terminated string with an unspecified encoding in an ELF file. To
/// avoid allocations we avoid trying to convert these to a String.
#[derive(Copy, Clone)]
pub struct StringView {
    reader: &'static Reader,
    offset: Offset,
}

impl StringView {
    pub fn new(reader: &'static Reader, offset: Offset) -> Self {
        StringView { reader, offset }
    }

    // pub fn new(
    //     reader: &'static Reader,
    //     offset: Offset,
    // ) -> Result<Self, Box<dyn std::error::Error>> {
    //     let mut len = 0;
    //     loop {
    //         let byte = reader.read_byte(offset + len)?;
    //         if byte == 0 {
    //             break;
    //         }
    //         len += 1;
    //     }

    //     Ok(StringView {
    //         reader,
    //         offset,
    //         len: len as usize,
    //     })
    // }
}

impl StringView {
    // Used when we try to print a string and discover that it is not utf-8. We can't
    // assume that the file encoding matches whatever the user is using so we just print
    // a '?' for high ASCII characters (and, in general, ELF files don't describe string
    // encodings).
    fn write_ascii(&self, f: &mut fmt::Formatter, mut i: Offset) -> fmt::Result {
        loop {
            match self.reader.read_byte(i) {
                Ok(byte) => {
                    if byte == 0 {
                        break;
                    }
                    if byte <= 0x7f {
                        write!(f, "{}", byte as char)?;
                    } else {
                        write!(f, "?")?;
                    }
                    i = i + 1;
                }
                Err(_) => {
                    write!(f, "[bad read]")?;
                    break;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Debug for StringView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for StringView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut i = self.offset;
        loop {
            match self.reader.read_byte(i) {
                Ok(byte) => {
                    if byte == 0 {
                        break;
                    }
                    if byte <= 0x7f {
                        write!(f, "{}", byte as char)?;
                        i = i + 1;
                    } else {
                        let len = if byte & 0b1111_0000 == 0b1111_0000 {
                            4
                        } else if byte & 0b1110_0000 == 0b1110_0000 {
                            3
                        } else if byte & 0b1100_0000 == 0b1100_0000 {
                            2
                        } else {
                            self.write_ascii(f, i)?;
                            break;
                        };
                        match self.reader.slice(i, len) {
                            Ok(v) => match str::from_utf8(v) {
                                Ok(s) => write!(f, "{s}")?,
                                Err(_) => {
                                    self.write_ascii(f, i)?;
                                    break;
                                }
                            },
                            Err(_) => {
                                write!(f, "[bad slice]")?;
                                break;
                            }
                        }
                        i = i + len as i64;
                    }
                }
                Err(_) => {
                    write!(f, "[bad read]")?;
                    break;
                }
            }
        }
        Ok(())
    }
}

impl Bytes<Offset> {
    pub fn from_raw(start: u64, size: usize) -> Self {
        Bytes {
            start: Offset::from_raw(start),
            size,
        }
    }
}

impl Bytes<VirtualAddr> {
    pub fn from_raw(start: u64, size: usize) -> Self {
        Bytes {
            start: VirtualAddr::from_raw(start),
            size,
        }
    }
}

impl<A: Add<i64, Output = A> + Copy + Ord> Bytes<A> {
    pub fn contains(&self, addr: A) -> bool {
        addr >= self.start && addr < self.end()
    }

    pub fn end(&self) -> A {
        self.start + (self.size as i64)
    }
}

impl Sub<Offset> for Offset {
    type Output = i64;

    fn sub(self, rhs: Offset) -> Self::Output {
        (self.0 as i64) - (rhs.0 as i64)
    }
}

impl VirtualAddr {
    pub fn from_raw(addr: u64) -> Self {
        VirtualAddr(addr)
    }
}

// impl RelativeAddr {
//     pub fn from_raw(addr: u64) -> Self {
//         RelativeAddr(addr)
//     }
// }

impl Offset {
    pub fn from_raw(addr: u64) -> Self {
        Offset(addr)
    }
}

impl Add<i64> for VirtualAddr {
    type Output = VirtualAddr;

    fn add(self, rhs: i64) -> Self::Output {
        if rhs < 0 {
            VirtualAddr(self.0 - (-rhs) as u64)
        } else {
            VirtualAddr(self.0 + rhs as u64)
        }
    }
}

impl Add<u64> for RelativeAddr {
    type Output = RelativeAddr;

    fn add(self, rhs: u64) -> Self::Output {
        RelativeAddr(self.0 + rhs)
    }
}

impl AddAssign<u64> for RelativeAddr {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Add<i64> for Offset {
    type Output = Offset;

    fn add(self, rhs: i64) -> Self::Output {
        if rhs < 0 {
            Offset(self.0 - (-rhs) as u64)
        } else {
            Offset(self.0 + rhs as u64)
        }
    }
}
