use alloc::vec::Vec;
use super::Result;

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut b = buf;
        while !b.is_empty() {
            let bytes_written = self.write(b)?;
            b = &b[bytes_written..];
        }
        Ok(())
    }
}

pub trait Read {
    // TODO: Implement EOF handling
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    // TODO: Implement EOF handling
    fn read_to_end(&mut self, into: &mut Vec<u8>) -> Result<()>;
}

#[derive(Copy, Clone, PartialEq)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64)
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
    fn seek_position(&mut self) -> Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
    fn seek_relative(&mut self, offset: i64) -> Result<u64> {
        self.seek(SeekFrom::Current(offset))
    }
}
