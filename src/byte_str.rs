use bytes::Bytes;
use nom::AsBytes;
use std::{ops, str};

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub(crate) struct ByteStr {
    bytes: Bytes,
}

impl ByteStr {
    #[inline]
    pub fn new() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    #[inline]
    pub const fn from_static(val: &'static str) -> ByteStr {
        ByteStr {
            bytes: Bytes::from_static(val.as_bytes()),
        }
    }

    #[inline]
    pub unsafe fn from_utf8_unchecked(bytes: Bytes) -> ByteStr {
        if cfg!(debug_assertions) {
            match str::from_utf8(&bytes) {
                Ok(_) => (),
                Err(e) => panic!(
                    "ByteStr::from_utf8_unchecked() with invalid bytes; error = {}, bytes = {:?}",
                    e, bytes
                ),
            }
        }

        ByteStr { bytes }
    }
}

impl ops::Deref for ByteStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        let b: &[u8] = self.bytes.as_ref();
        unsafe { str::from_utf8_unchecked(b) }
    }
}

impl From<String> for ByteStr {
    fn from(value: String) -> Self {
        ByteStr {
            bytes: Bytes::from(value),
        }
    }
}

impl<'a> From<&'a str> for ByteStr {
    fn from(value: &'a str) -> Self {
        ByteStr {
            bytes: Bytes::copy_from_slice(value.as_bytes()),
        }
    }
}

impl From<ByteStr> for Bytes {
    fn from(value: ByteStr) -> Self {
        value.bytes
    }
}
