use core::str::Utf8Error;

use bytes::Buf;

use crate::{
    CompactStr,
    Repr,
};

impl CompactStr {
    /// Converts a buffer of bytes to a [`CompactStr`]
    ///
    /// # Examples
    /// ### Basic usage
    /// ```
    /// # use compact_str::CompactStr;
    /// # use std::collections::VecDeque;
    ///
    /// // `bytes::Buf` is implemented for `VecDeque<u8>`
    /// let mut sparkle_heart = VecDeque::from(vec![240, 159, 146, 150]);
    /// // We know these bytes are valid, so we can `.unwrap()` or `.expect(...)` here
    /// let compact_str = CompactStr::from_utf8_buf(&mut sparkle_heart).expect("valid utf-8");
    ///
    /// assert_eq!(compact_str, "💖");
    /// ```
    ///
    /// ### With invalid/non-UTF8 bytes
    /// ```
    /// # use compact_str::CompactStr;
    /// # use std::io;
    ///
    /// // `bytes::Buf` is implemented for `std::io::Cursor<&[u8]>`
    /// let mut invalid = io::Cursor::new(&[0, 159]);
    ///
    /// // The provided buffer is invalid, so trying to create a `ComapctStr` will fail
    /// assert!(CompactStr::from_utf8_buf(&mut invalid).is_err());
    /// ```
    pub fn from_utf8_buf<B: Buf>(buf: &mut B) -> Result<Self, Utf8Error> {
        Repr::from_utf8_buf(buf).map(|repr| CompactStr { repr })
    }

    /// Converts a buffer of bytes to a [`CompactStr`], without checking that the provided buffer is
    /// valid UTF-8.
    ///
    /// # Safety
    /// This function is unsafe because it does not check that the provided bytes are valid UTF-8.
    /// If this constraint is violated, it may cause memory safety issues with futures uses of the
    /// `ComapctStr`, as the rest of the library assumes that `CompactStr`s are valid UTF-8
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactStr;
    /// # use std::io;
    ///
    /// let word = "hello world";
    /// // `bytes::Buf` is implemented for `std::io::Cursor<&[u8]>`
    /// let mut buffer = io::Cursor::new(word.as_bytes());
    /// let compact_str = unsafe { CompactStr::from_utf8_buf_unchecked(&mut buffer) };
    ///
    /// assert_eq!(compact_str, word);
    /// ```
    pub unsafe fn from_utf8_buf_unchecked<B: Buf>(buf: &mut B) -> Self {
        let repr = Repr::from_utf8_buf_unchecked(buf);
        CompactStr { repr }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::tests::{
        rand_bytes,
        rand_unicode,
    };
    use crate::CompactStr;

    const MAX_SIZE: usize = core::mem::size_of::<String>();

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_buffers_roundtrip(#[strategy(rand_unicode())] word: String) {
        let mut buf = Cursor::new(word.as_bytes());
        let compact = CompactStr::from_utf8_buf(&mut buf).unwrap();

        proptest::prop_assert_eq!(&word, &compact);
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_allocated_properly(#[strategy(rand_unicode())] word: String) {
        let mut buf = Cursor::new(word.as_bytes());
        let compact = CompactStr::from_utf8_buf(&mut buf).unwrap();

        if word.len() <= MAX_SIZE {
            proptest::prop_assert!(!compact.is_heap_allocated())
        } else {
            proptest::prop_assert!(compact.is_heap_allocated())
        }
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_only_accept_valid_utf8(#[strategy(rand_bytes())] bytes: Vec<u8>) {
        let mut buf = Cursor::new(bytes.as_slice());

        let compact_result = CompactStr::from_utf8_buf(&mut buf);
        let str_result = core::str::from_utf8(bytes.as_slice());

        match (compact_result, str_result) {
            (Ok(c), Ok(s)) => prop_assert_eq!(c, s),
            (Err(c_err), Err(s_err)) => prop_assert_eq!(c_err, s_err),
            _ => panic!("CompactStr and core::str read UTF-8 differently?"),
        }
    }
}
