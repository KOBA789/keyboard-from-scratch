use core::cmp;
use core::convert::Into;

#[derive(Debug)]
pub struct ReadCursor<'a> {
    buf: &'a mut [u8],
    len: usize,
    pos: usize,
}

impl<'a> ReadCursor<'a> {
    #[allow(dead_code)]
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self::with_len(buf, buf.len())
    }

    pub fn with_len(buf: &'a mut [u8], len: usize) -> Self {
        debug_assert!(buf.len() >= len);
        ReadCursor { buf, len, pos: 0 }
    }

    pub fn read<'b>(&'b mut self, len: usize) -> &'b [u8]
    where
        'a: 'b,
    {
        let len = cmp::min(len, self.rest());
        let start = self.pos();
        self.pos += len;
        &self.buf[start..self.pos]
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn rest(&self) -> usize {
        self.len() - self.pos()
    }

    pub fn into_buf(self) -> &'a mut [u8] {
        self.buf
    }
}
impl<'a> Into<&'a mut [u8]> for ReadCursor<'a> {
    fn into(self) -> &'a mut [u8] {
        self.into_buf()
    }
}

#[derive(Debug)]
pub struct WriteCursor<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> WriteCursor<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        WriteCursor { buf, len: 0 }
    }

    pub fn write(&mut self, src: &[u8]) -> usize {
        let len = cmp::min(src.len(), self.buf.len() - self.len());
        let start = self.len();
        self.len += len;

        let src = &src[0..len];
        let dst = &mut self.buf[start..self.len];
        dst.copy_from_slice(src);
        len
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[allow(dead_code)]
    pub fn as_slice(&'a self) -> &'a [u8] {
        &self.buf[0..self.len]
    }

    pub fn into_buf(self) -> &'a mut [u8] {
        self.buf
    }

    pub fn into_read(self) -> ReadCursor<'a> {
        let len = self.len();
        ReadCursor::with_len(self.into_buf(), len)
    }
}
impl<'a> Into<&'a mut [u8]> for WriteCursor<'a> {
    fn into(self) -> &'a mut [u8] {
        self.into_buf()
    }
}
