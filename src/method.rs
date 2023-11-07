use self::extension::{AllocatedExtension, InlineExtension};
use core::fmt;
use std::{error::Error, str::FromStr};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Method(Inner);
pub struct InvalidMethod {
    _priv: (),
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum Inner {
    Head,
    Options,
    Get,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
    Patch,
    ExtensionInline(InlineExtension),
    ExtensionAllocated(AllocatedExtension),
}

use Inner::*;
impl Method {
    /// GET
    pub const GET: Method = Method(Get);

    /// POST
    pub const POST: Method = Method(Post);

    /// PUT
    pub const PUT: Method = Method(Put);

    /// DELETE
    pub const DELETE: Method = Method(Delete);

    /// HEAD
    pub const HEAD: Method = Method(Head);

    /// OPTIONS
    pub const OPTIONS: Method = Method(Options);

    /// CONNECT
    pub const CONNECT: Method = Method(Connect);

    /// PATCH
    pub const PATCH: Method = Method(Patch);

    /// TRACE
    pub const TRACE: Method = Method(Trace);

    pub fn from_bytes(src: &[u8]) -> Result<Method, InvalidMethod> {
        match src.len() {
            0 => Err(InvalidMethod::new()),
            3 => match src {
                b"GET" => Ok(Method(Get)),
                b"PUT" => Ok(Method(Put)),
                _ => Method::extension_inline(src),
            },
            4 => match src {
                b"POST" => Ok(Method(Post)),
                b"HEAD" => Ok(Method(Head)),
                _ => Method::extension_inline(src),
            },
            5 => match src {
                b"PATCH" => Ok(Method(Patch)),
                b"TRACE" => Ok(Method(Trace)),
                _ => Method::extension_inline(src),
            },
            6 => match src {
                b"DELETE" => Ok(Method(Delete)),
                _ => Method::extension_inline(src),
            },
            7 => match src {
                b"OPTIONS" => Ok(Method(Options)),
                b"CONNECT" => Ok(Method(Connect)),
                _ => Method::extension_inline(src),
            },
            _ => {
                if src.len() < InlineExtension::MAX {
                    Method::extension_inline(src)
                } else {
                    let allocated = AllocatedExtension::new(src)?;

                    Ok(Method(ExtensionAllocated(allocated)))
                }
            }
        }
    }

    fn extension_inline(src: &[u8]) -> Result<Method, InvalidMethod> {
        let inline = InlineExtension::new(src)?;
        Ok(Method(Inner::ExtensionInline(inline)))
    }

    pub fn is_safe(&self) -> bool {
        match self.0 {
            Inner::Get | Inner::Head | Inner::Options | Inner::Trace => true,
            _ => false,
        }
    }

    pub fn is_idempotent(&self) -> bool {
        match self.0 {
            Inner::Put | Inner::Delete => true,
            _ => self.is_safe(),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        match self.0 {
            Inner::Head => "HEAD",
            Inner::Options => "OPTIONS",
            Inner::Get => "GET",
            Inner::Post => "POST",
            Inner::Put => "PUT",
            Inner::Delete => "DELETE",
            Inner::Trace => "TRACE",
            Inner::Connect => "CONNECT",
            Inner::Patch => "PATCH",
            Inner::ExtensionInline(ref inline) => inline.as_str(),
            Inner::ExtensionAllocated(ref allocated) => allocated.as_str(),
        }
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> PartialEq<&'a Method> for Method {
    #[inline]
    fn eq(&self, other: &&'a Method) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<Method> for &'a Method {
    #[inline]
    fn eq(&self, other: &Method) -> bool {
        *self == other
    }
}

impl PartialEq<str> for Method {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Method> for str {
    #[inline]
    fn eq(&self, other: &Method) -> bool {
        self == other.as_ref()
    }
}

impl<'a> PartialEq<&'a str> for Method {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

impl<'a> PartialEq<Method> for &'a str {
    #[inline]
    fn eq(&self, other: &Method) -> bool {
        *self == other.as_ref()
    }
}

impl fmt::Debug for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl Default for Method {
    fn default() -> Self {
        Method::GET
    }
}

impl<'a> From<&'a Method> for Method {
    fn from(value: &'a Method) -> Self {
        value.clone()
    }
}

impl<'a> TryFrom<&'a [u8]> for Method {
    type Error = InvalidMethod;
    #[inline]
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Method::from_bytes(value)
    }
}

impl<'a> TryFrom<&'a str> for Method {
    type Error = InvalidMethod;
    #[inline]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        TryFrom::try_from(value.as_bytes())
    }
}

impl FromStr for Method {
    type Err = InvalidMethod;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TryFrom::try_from(s)
    }
}

impl InvalidMethod {
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl fmt::Debug for InvalidMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InvalidMethod").finish()
    }
}

impl fmt::Display for InvalidMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid HTTP method")
    }
}

impl Error for InvalidMethod {}

mod extension {
    use super::InvalidMethod;
    use std::str;

    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct InlineExtension([u8; InlineExtension::MAX], u8);
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct AllocatedExtension(Box<[u8]>);

    impl InlineExtension {
        pub const MAX: usize = 15;

        pub fn new(src: &[u8]) -> Result<InlineExtension, InvalidMethod> {
            let mut data: [u8; InlineExtension::MAX] = Default::default();

            write_checked(src, &mut data)?;

            Ok(InlineExtension(data, src.len() as u8))
        }

        pub fn as_str(&self) -> &str {
            let InlineExtension(ref data, len) = self;
            unsafe { str::from_utf8_unchecked(&data[..*len as usize]) }
        }
    }

    impl AllocatedExtension {
        pub fn new(src: &[u8]) -> Result<AllocatedExtension, InvalidMethod> {
            let mut data: Vec<u8> = vec![0; src.len()];

            write_checked(src, &mut data)?;

            Ok(AllocatedExtension(data.into_boxed_slice()))
        }

        pub fn as_str(&self) -> &str {
            unsafe { str::from_utf8_unchecked(&self.0) }
        }
    }

    const METHOD_CHARS: [u8; 256] = [
        //  0      1      2      3      4      5      6      7      8      9
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', //   x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', //  1x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', //  2x
        b'\0', b'\0', b'\0', b'!', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', //  3x
        b'\0', b'\0', b'*', b'+', b'\0', b'-', b'.', b'\0', b'0', b'1', //  4x
        b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'\0', b'\0', //  5x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'A', b'B', b'C', b'D', b'E', //  6x
        b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', //  7x
        b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', //  8x
        b'Z', b'\0', b'\0', b'\0', b'^', b'_', b'`', b'a', b'b', b'c', //  9x
        b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', // 10x
        b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', // 11x
        b'x', b'y', b'z', b'\0', b'|', b'\0', b'~', b'\0', b'\0', b'\0', // 12x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 13x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 14x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 15x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 16x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 17x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 18x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 19x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 20x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 21x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 22x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 23x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 24x
        b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 25x
    ];

    fn write_checked(src: &[u8], dst: &mut [u8]) -> Result<(), InvalidMethod> {
        for (i, &b) in src.iter().enumerate() {
            let b = METHOD_CHARS[b as usize];

            if b == 0 {
                return Err(InvalidMethod::new());
            }

            dst[i] = b;
        }
        Ok(())
    }
}
