use std::ops::{Add, Sub};

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
