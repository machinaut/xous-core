use core::fmt::{Error, Write};
use core::slice;
use xous;

pub struct LogStr<'a> {
    raw_slice: &'a mut [u8],
    len: usize,
    string: &'a str,
}

impl<'a> LogStr<'a> {
    pub fn new() -> LogStr<'a> {
        let mem = xous::syscall::map_memory(
            None,
            None,
            4096,
            xous::MemoryFlags::R | xous::MemoryFlags::W,
        )
        .expect("couldn't allocate memory");

        let raw_slice = unsafe { slice::from_raw_parts_mut(mem.base, 4096) };

        LogStr {
            raw_slice,
            len: 0,
            string: unsafe {
                core::str::from_utf8_unchecked(slice::from_raw_parts(mem.base, 0))
            },
        }
    }

    pub fn into_memory_message(self, id: xous::MessageId) -> Result<xous::MemoryMessage, xous::Error> {
        Ok(xous::MemoryMessage {
            id: id,
            buf: xous::MemoryAddress::new(self.raw_slice.as_ptr() as usize),
            buf_size: xous::MemorySize::new(self.raw_slice.len()),
            offset: None,
            valid: xous::MemorySize::new(self.len),
        })
    }
}

impl<'a> Write for LogStr<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.raw_slice[self.len] = c;
            self.len += 1;
        }
        self.string = unsafe { core::str::from_utf8_unchecked(slice::from_raw_parts(self.raw_slice.as_ptr(), self.len)) };
        Ok(())
    }
}
