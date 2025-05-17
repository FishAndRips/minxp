use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::collections::TryReserveError;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use core::str::FromStr;

/// OsString
///
/// This is not guaranteed to have the same binary representation as Rust's standard library even on
/// Windows.
#[derive(Clone, PartialEq, Ord, PartialOrd, Eq)]
#[repr(transparent)]
pub struct OsString {
    string: String
}

impl Debug for OsString {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.string, f)
    }
}

impl Debug for OsStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl OsString {
    pub fn new() -> Self {
        Self {
            string: String::new()
        }
    }

    pub unsafe fn from_encoded_bytes_unchecked(bytes: &[u8]) -> Self {
        Self {
            string: unsafe { core::str::from_utf8_unchecked(bytes).to_string() }
        }
    }

    pub fn as_os_str(&self) -> &OsStr {
        OsStr::from_str(self.string.as_str())
    }

    pub fn into_encoded_bytes(self) -> Vec<u8> {
        self.string.into_bytes()
    }

    pub fn into_string(self) -> Result<String, OsString> {
        // This is infallible
        Ok(self.string)
    }

    pub fn push<S: AsRef<OsStr>>(&mut self, what: S) {
        self.string += &what.as_ref().inner
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            string: String::with_capacity(capacity)
        }
    }

    pub fn clear(&mut self) {
        self.string.clear();
    }

    pub fn capacity(&self) -> usize {
        self.string.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.string.reserve(additional);
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.string.try_reserve(additional)
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.string.reserve_exact(additional);
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.string.try_reserve_exact(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.string.shrink_to_fit();
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.string.shrink_to(min_capacity);
    }

    pub fn into_boxed_os_str(self) -> Box<OsStr> {
        let s = Box::into_raw(self.string.into_boxed_str());
        unsafe { Box::from_raw(s as *mut OsStr) }
    }

    pub fn to_str(&self) -> Option<&str> {
        Some(self.string.as_str())
    }

    pub(crate) fn from_str(str: &str) -> OsString {
        Self {
            string: str.to_string()
        }
    }

    pub(crate) fn truncate(&mut self, new_len: usize) {
        self.string.truncate(new_len)
    }
}

impl FromStr for OsString {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(OsString::from_str(s))
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct OsStr {
    inner: str
}

impl OsStr {
    pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> &Self {
        s.as_ref()
    }

    pub unsafe fn from_encoded_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { Self::new(core::str::from_utf8_unchecked(bytes)) }
    }

    pub fn to_str(&self) -> Option<&str> {
        Some(&self.inner)
    }

    pub(crate) fn from_str(mut s: &str) -> &Self {
        if let Some(nul) = s.find("\x00") {
            s = &s[..nul];
        };

        // SAFETY: OsStr is just str
        unsafe { &*(s as *const str as *const OsStr) }
    }

    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.inner)
    }

    pub fn to_os_string(&self) -> OsString {
        OsString::from_str(&self.inner)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn into_os_string(self: Box<OsStr>) -> OsString {
        let s = Box::into_raw(self);
        let s = unsafe { Box::from_raw(s as *mut str) };
        s.into_string().into()
    }

    pub fn as_encoded_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    pub fn make_ascii_lowercase(&mut self) {
        self.inner.make_ascii_lowercase()
    }

    pub fn make_ascii_uppercase(&mut self) {
        self.inner.make_ascii_uppercase()
    }

    pub fn to_ascii_lowercase(&self) -> OsString {
        self.inner.to_ascii_lowercase().into()
    }

    pub fn to_ascii_uppercase(&self) -> OsString {
        self.inner.to_ascii_uppercase().into()
    }

    pub fn is_ascii(&self) -> bool {
        self.inner.is_ascii()
    }

    pub fn display(&self) -> impl Display {
        self.as_str()
    }

    pub fn eq_ignore_ascii_case<S: AsRef<OsStr>>(&self, other: S) -> bool {
        self.inner.eq_ignore_ascii_case(&other.as_ref().inner)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.inner
    }
}

impl AsRef<OsStr> for str {
    fn as_ref(&self) -> &OsStr {
        OsStr::from_str(self)
    }
}

impl AsRef<OsStr> for String {
    fn as_ref(&self) -> &OsStr {
        OsStr::from_str(self)
    }
}

impl AsRef<OsStr> for OsString {
    fn as_ref(&self) -> &OsStr {
        OsStr::from_str(self.string.as_str())
    }
}

impl AsRef<OsStr> for OsStr {
    fn as_ref(&self) -> &OsStr {
        self
    }
}

impl From<String> for OsString {
    fn from(value: String) -> Self {
        Self { string: value }
    }
}

impl Deref for OsString {
    type Target = OsStr;
    fn deref(&self) -> &Self::Target {
        self.as_os_str()
    }
}

impl ToOwned for OsStr {
    type Owned = OsString;
    fn to_owned(&self) -> Self::Owned {
        self.to_os_string()
    }
}

impl Borrow<OsStr> for OsString {
    fn borrow(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl PartialEq<OsStr> for OsString {
    fn eq(&self, other: &OsStr) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<OsString> for OsStr {
    fn eq(&self, other: &OsString) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for OsStr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<OsStr> for str {
    fn eq(&self, other: &OsStr) -> bool {
        self == other.as_str()
    }
}

impl From<&OsStr> for Arc<OsStr> {
    fn from(value: &OsStr) -> Self {
        value.to_os_string().into_boxed_os_str().into()
    }
}

impl From<&mut OsStr> for Arc<OsStr> {
    fn from(value: &mut OsStr) -> Self {
        Arc::from(&*value)
    }
}
