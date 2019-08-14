use bytes::{Buf, BufMut};
use mc_varint::{VarIntRead, VarIntWrite, VarLongRead, VarLongWrite};
use std::{
    i32,
    io::{ErrorKind, Read, Result},
    usize,
};

pub trait VarReadExt {
    fn read_var_i32(&mut self) -> Result<i32>;

    fn read_var_i64(&mut self) -> Result<i64>;

    fn read_str(&mut self) -> Result<String>;

    fn read_var_len(&mut self) -> Result<usize> {
        let len = self.read_var_i32()?;
        match len {
            x if x >= 0 => Ok(x as usize),
            _ => Err(ErrorKind::InvalidData.into()),
        }
    }
}

pub trait VarWriteExt {
    fn write_var_i32(&mut self, val: i32) -> Result<()>;

    fn write_var_i64(&mut self, val: i64) -> Result<()>;

    fn write_str(&mut self, data: &str) -> Result<()>;

    fn write_var_len(&mut self, len: usize) -> Result<()> {
        if len > (i32::MAX as usize) {
            return Err(ErrorKind::InvalidData.into());
        }

        self.write_var_i32(len as i32)
    }
}

impl<B: Buf> VarReadExt for B {
    fn read_var_i32(&mut self) -> Result<i32> {
        Ok(self.by_ref().reader().read_var_int()?.into())
    }

    fn read_var_i64(&mut self) -> Result<i64> {
        Ok(self.by_ref().reader().read_var_long()?.into())
    }

    fn read_str(&mut self) -> Result<String> {
        let length = self.read_var_len()?;

        if self.remaining() < length {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        let mut res = String::with_capacity(length);
        self.by_ref()
            .take(length)
            .reader()
            .read_to_string(&mut res)?;
        Ok(res)
    }
}

impl<B: BufMut> VarWriteExt for B {
    fn write_var_i32(&mut self, val: i32) -> Result<()> {
        // Double cast to prevent sign-extension
        if self.remaining_mut() < var_i32_length(val) {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        self.by_ref().writer().write_var_int(val.into())
    }

    fn write_var_i64(&mut self, val: i64) -> Result<()> {
        if self.remaining_mut() < var_i64_length(val) {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        self.by_ref().writer().write_var_long(val.into())
    }

    fn write_str(&mut self, data: &str) -> Result<()> {
        if self.remaining_mut() < var_usize_length(data.len()) + data.len() {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        self.write_var_len(data.len())?;
        self.put(data);
        Ok(())
    }
}

pub fn var_i32_length(val: i32) -> usize {
    // Prevent sign-extension during shift
    var_u64_length(u64::from(val as u32))
}

pub fn var_i64_length(val: i64) -> usize {
    // Prevent sign-extension during shift
    var_u64_length(val as u64)
}

pub fn var_usize_length(val: usize) -> usize {
    var_u64_length(val as u64)
}

pub fn var_u64_length(mut val: u64) -> usize {
    let mut count = 0;
    while val != 0 {
        count += 1;
        val >>= 7;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn var_length() {
        assert_eq!(var_i32_length(1), 1);
        assert_eq!(var_i32_length(2147483647), 5);
        assert_eq!(var_i32_length(-1), 5);

        assert_eq!(var_i64_length(1), 1);
        assert_eq!(var_i64_length(2147483647), 5);
        assert_eq!(var_i64_length(-1), 10);
        assert_eq!(var_i64_length(9223372036854775807), 9)
    }

    #[test]
    fn var_read() {
        let data = [
            0x7fu8, 0x80, 0x01, 0xff, 0xff, 0xff, 0xff, 0x07, 0xff, 0xff, 0xff,
            0xff, 0x0f,
        ];
        let mut cur = Cursor::new(&data);

        assert_eq!(cur.read_var_i32().unwrap(), 127);
        assert_eq!(cur.read_var_i32().unwrap(), 128);
        assert_eq!(cur.read_var_i32().unwrap(), 2147483647);
        assert_eq!(cur.read_var_i32().unwrap(), -1);
    }

    #[test]
    fn var_write() {
        let mut data = Vec::new();

        data.write_var_i32(127).unwrap();
        assert_eq!(&data, &[0x7f]);
        data.clear();

        data.write_var_i32(-1).unwrap();
        assert_eq!(&data, &[0xff, 0xff, 0xff, 0xff, 0x0f]);
        data.clear();
    }
}
