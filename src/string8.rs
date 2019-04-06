use std::{convert::TryFrom, fmt};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct String8([u8; 8]);

impl String8 {
    pub fn from_raw_parts(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    pub fn from_str(s: &str) -> Result<Self, IntoString8Error> {
        Self::from_bytes(s.as_bytes())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, IntoString8Error> {
        if bytes.len() > 8 {
            Err(IntoString8Error::Len(bytes.len()))
        } else if let Some(p) = bytes
            .into_iter()
            .rev()
            .skip_while(|b| **b == 0)
            .position(|b| *b == 0)
        {
            Err(IntoString8Error::Nul(p))
        } else {
            Ok(Self::from_bytes_unchecked(bytes))
        }
    }

    pub fn from_str_unchecked(s: &str) -> Self {
        Self::from_bytes_unchecked(s.as_bytes())
    }

    pub fn from_bytes_unchecked(bytes: &[u8]) -> Self {
        let mut arr: [u8; 8] = Default::default();
        arr[..usize::min(8, bytes.len())].copy_from_slice(bytes);
        Self(arr)
    }
}

#[derive(Debug)]
pub enum IntoString8Error {
    Nul(usize),
    Len(usize),
}

impl fmt::Display for IntoString8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntoString8Error::Nul(p) => write!(f, "Inner null byte at position {}", p),
            IntoString8Error::Len(p) => write!(f, "String is longer than 8 bytes ({} bytes)", p),
        }
    }
}

impl std::error::Error for IntoString8Error {}

impl TryFrom<&str> for String8 {
    type Error = IntoString8Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::from_str(s)
    }
}

impl TryFrom<&[u8]> for String8 {
    type Error = IntoString8Error;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(s)
    }
}
