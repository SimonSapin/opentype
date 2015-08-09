//! Primitive data types.

#![allow(non_snake_case)]

use std::io::Read;
use std::{mem, ptr};

use Result;
use band::{Atom, Band};

/// A 32-bit signed fixed-point number.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Fixed(pub u32);

/// A date in seconds since January 1, 1904.
pub type LONGDATETIME = i64;

/// A 16-bit signed integer.
pub type SHORT = i16;

/// A 32-bit unsigned integer.
pub type ULONG = u32;

/// A 16-bit unsigned integer.
pub type USHORT = u16;

impl Fixed {
    #[cfg(test)]
    pub fn as_f32(&self) -> f32 {
        ((self.0 as f32) * 0.0000152587890625 * 1000.0).round() / 1000.0
    }
}

#[cfg(target_endian = "big")]
macro_rules! convert(
    ($data:ident) => ();
);

#[cfg(target_endian = "little")]
macro_rules! convert(
    ($data:ident) => ($data.reverse());
);

macro_rules! read(
    ($band:ident, $count:expr) => (unsafe {
        let mut buffer: [u8; $count] = mem::uninitialized();
        if try!($band.read(&mut buffer)) != $count {
            return raise!("failed to read as much as needed");
        }
        convert!(buffer);
        Ok(ptr::read(buffer.as_ptr() as *const _))
    });
);

impl Atom for Fixed {
    fn read<T: Band>(band: &mut T) -> Result<Self> {
        read!(band, 4)
    }
}

impl Atom for LONGDATETIME {
    fn read<T: Band>(band: &mut T) -> Result<Self> {
        read!(band, 8)
    }
}

impl Atom for SHORT {
    fn read<T: Band>(band: &mut T) -> Result<Self> {
        read!(band, 2)
    }
}

impl Atom for ULONG {
    fn read<T: Band>(band: &mut T) -> Result<Self> {
        read!(band, 4)
    }
}

impl Atom for USHORT {
    fn read<T: Band>(band: &mut T) -> Result<Self> {
        read!(band, 2)
    }
}
