use std::io;

use super::{Error, Result};

macro_rules! read_num {
    ($fn:ident, $ty:ty, $length:expr) => {
        fn $fn(&mut self) -> Result<$ty> {
            let mut buf = [0; $length];
            self.read_exact(&mut buf)?;
            Ok(<$ty>::from_le_bytes(buf))
        }
    };
}

pub trait ReadExt: io::Read {
    read_num!(read_u8, u8, 1);
    read_num!(read_u16, u16, 2);
    read_num!(read_u32, u32, 4);
    read_num!(read_u64, u64, 8);

    read_num!(read_i8, i8, 1);
    read_num!(read_i16, i16, 2);
    read_num!(read_i32, i32, 4);
    read_num!(read_i64, i64, 8);

    read_num!(read_f64, f64, 8);

    fn read_bool(&mut self) -> Result<bool> {
        let val = self.read_u8()?;
        if val > 1 {
            Err(Error::InvalidBoolValue)
        } else {
            Ok(val != 0)
        }
    }
}

impl<R: io::Read + ?Sized> ReadExt for R {}
