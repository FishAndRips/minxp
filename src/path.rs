use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use crate::ffi::{OsStr, OsString};
use crate::fs::*;
use alloc::collections::TryReserveError;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use crate::env::current_dir;

#[must_use]
pub fn is_separator(c: char) -> bool {
    c == MAIN_SEPARATOR || c == '/'
}

pub const MAIN_SEPARATOR: char = '\\';
pub const MAIN_SEPARATOR_STR: &str = "\\";

/// Base max path
pub(crate) const MAX_PATH: usize = windows_sys::Win32::Foundation::MAX_PATH as usize;

/// Extended max path
pub(crate) const MAX_PATH_EXTENDED: usize = 32767 as usize;

#[derive(Clone, PartialEq, Ord, PartialOrd, Eq)]
#[repr(transparent)]
pub struct PathBuf {
    data: OsString
}

impl Debug for PathBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.data, f)
    }
}

impl Debug for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl PathBuf {
    pub fn new() -> PathBuf {
        Self { data: OsString::new() }
    }
    pub fn with_capacity(capacity: usize) -> PathBuf {
        Self { data: OsString::with_capacity(capacity) }
    }
    pub fn as_path(&self) -> &Path {
        self
    }
    pub fn push<S: AsRef<Path>>(&mut self, part: S) {
        let part = part.as_ref();

        if part.is_absolute() || part.has_drive_letter() {
            self.data.clear();
            self.data.push(part.as_os_str());
            return
        }

        if !self.data.as_str().chars().next_back().is_some_and(is_separator) {
            self.data.push(MAIN_SEPARATOR_STR);
        }
        self.data.push(part.as_os_str());
    }
    pub fn pop(&mut self) -> bool {
        match self.parent() {
            Some(s) => {
                self.data.truncate(s.as_os_str().len());
                true
            }
            None => false
        }
    }
    pub fn set_file_name<S: AsRef<OsStr>>(&mut self, file_name: S) {
        self.truncate_extraneous_suffixes();

        if self.file_name().is_some() {
            self.pop();
        }
        self.push(file_name.as_ref());
    }
    pub fn set_extension<S: AsRef<OsStr>>(&mut self, extension: S) -> bool {
        if !self.file_name().is_some() {
            return false
        }

        self.truncate_extraneous_suffixes();

        match self.extension() {
            Some(e) => self.data.truncate(self.data.len() - e.len()),
            None => self.data.push(".")
        }
        self.data.push(extension);

        true
    }
    fn truncate_extraneous_suffixes(&mut self) {
        self.data.truncate(self.remove_extraneous_suffixes().path_len())
    }
    pub fn as_mut_os_string(&mut self) -> &mut OsString {
        &mut self.data
    }
    pub fn into_os_string(self) -> OsString {
        self.data
    }
    pub fn into_boxed_path(self) -> Box<Path> {
        let s = Box::into_raw(self.data.into_boxed_os_str());
        unsafe { Box::from_raw(s as *mut Path) }
    }
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    pub fn clear(&mut self) {
        self.data.clear()
    }
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional)
    }
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.data.try_reserve(additional)
    }
    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional)
    }
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.data.try_reserve_exact(additional)
    }
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit()
    }
    pub fn shrink_to(&mut self, amount: usize) {
        self.data.shrink_to(amount)
    }
    pub(crate) fn from_string(mut string: String) -> Self {
        // TODO: Discombobulate forward slashes, too
        if string.starts_with(r#"\\?\"#) {
            string.remove(0);
            string.remove(0);
            string.remove(0);
            string.remove(0);
        }
        Self { data: string.into() }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Path {
    inner: OsStr
}

impl Path {
    pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> &Path {
        let mut s = s.as_ref().as_str();
        if s.starts_with(r#"\\?\"#) {
            s = &s[4..];
        }
        Self::from_os_str(s.as_ref())
    }

    pub fn as_os_str(&self) -> &OsStr {
        &self.inner
    }

    pub fn as_mut_os_str(&mut self) -> &mut OsStr {
        &mut self.inner
    }

    pub fn to_str(&self) -> Option<&str> {
        self.inner.to_str()
    }

    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        self.inner.to_string_lossy()
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf { data: self.inner.to_os_string() }
    }

    pub fn is_absolute(&self) -> bool {
        if self.has_drive_letter() && self.as_os_str().as_str().chars().skip(2).next().is_some_and(is_separator) {
            return true
        }

        let mut p = self.iter();

        // empty paths are not absolute
        let Some(first) = p.next() else { return false };
        let first = first.as_str();

        // Starts with backslash
        if !first.is_empty() {
            return false
        }

        // Double backslash?
        let Some(next) = p.next() else { return false };
        if !next.is_empty() {
            return false
        }

        // Triple backslash???
        let Some(next) = p.next() else { return false };
        if next.is_empty() {
            return false
        }

        // Formatted as "\\Something\..."
        p.next().is_some()
    }

    fn has_drive_letter(&self) -> bool {
        let mut p = self.iter();

        let Some(first) = p.next() else { return false };
        let first = first.as_str();
        let mut chars = first.chars();

        // Stars with a character and a colon, and then either the end of the string or a backslash
        let [Some(drive_letter), Some(':')] = [chars.next(), chars.next()] else {
            return false
        };

        if !drive_letter.is_ascii_alphabetic() {
            return false
        }

        drive_letter.is_ascii_alphabetic()
    }

    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    pub fn has_root(&self) -> bool {
        self.is_absolute() || self.as_os_str().as_str().chars().next().is_some_and(is_separator)
    }

    pub fn parent(&self) -> Option<&Path> {
        let mut i = self.as_relative_to_root().components();
        let current = i.next_back()?;

        unsafe {
            match i.next_back() {
                Some(n) => Some(self.trim_to_end_of(n.as_ref())),
                None => Some(self.trim_to_start_of(current.as_ref()))
            }
        }
    }

    pub fn ancestors(&self) -> Ancestors {
        Ancestors { path: self }
    }

    pub fn file_name(&self) -> Option<&OsStr> {
        let final_component = self.as_relative_to_root().components().next_back()?;
        (final_component.as_str() != "..").then_some(final_component)
    }

    pub fn strip_prefix<P: AsRef<Path>>(&self, base: P) -> Result<&Path, StripPrefixError> {
        let mut a = self.components();
        let mut b = base.as_ref().components();

        loop {
            let Some(n) = a.next() else { return Err(StripPrefixError {}) };
            // SAFETY: part of the string
            let Some(m) = b.next() else { return Ok(unsafe { self.rtrim_to_start_of(n.as_ref()) }) };
            if n != m {
                return Err(StripPrefixError {})
            }
        }
    }

    pub fn starts_with<P: AsRef<Path>>(&self, base: P) -> bool {
        let mut a = self.components();
        let mut b = base.as_ref().components();

        loop {
            let Some(n) = a.next() else { return b.next().is_none() };
            let Some(m) = b.next() else { return true };
            if n != m {
                return false
            }
        }
    }

    pub fn ends_with<P: AsRef<Path>>(&self, child: P) -> bool {
        let mut a = self.components();
        let mut b = child.as_ref().components();

        loop {
            let Some(n) = a.next_back() else { return b.next_back().is_none() };
            let Some(m) = b.next_back() else { return true };
            if n != m {
                return false
            }
        }
    }

    pub fn file_stem(&self) -> Option<&OsStr> {
        self.split_file_stem_extension().map(|i| i.0)
    }

    pub fn extension(&self) -> Option<&OsStr> {
        self.split_file_stem_extension().map(|i| i.1).flatten()
    }

    fn split_file_stem_extension(&self) -> Option<(&OsStr, Option<&OsStr>)> {
        let file_name = self.file_name()?.as_str();
        let Some(dot) = file_name.rfind(".") else { return Some((file_name.as_ref(), None)) };
        let (stem, extension) = file_name.split_at(dot);

        if stem.is_empty() {
            Some((file_name.as_ref(), None))
        }
        else {
            let extension = &extension[1..]; // exclude dot
            Some((stem.as_ref(), Some(extension.as_ref())))
        }
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let mut new_path = self.to_owned();
        new_path.push(path);
        new_path
    }

    pub fn with_file_name<S: AsRef<OsStr>>(&self, file_name: S) -> PathBuf {
        let mut p = self.to_owned();
        p.set_file_name(file_name);
        p
    }

    pub fn with_extension<S: AsRef<OsStr>>(&self, extension: S) -> PathBuf {
        let mut p = self.to_owned();
        p.set_extension(extension);
        p
    }

    pub fn components(&self) -> Components {
        Components { iter: self.iter() }
    }

    pub fn iter(&self) -> Iter {
        let string = self.inner.as_str();
        Iter {
            iter: string.split(is_separator),
            original_string: string,
            iterated_amount: 0
        }
    }

    pub fn display(&self) -> impl Display {
        self.inner.as_str()
    }

    pub fn metadata(&self) -> crate::io::Result<Metadata> {
        metadata(self)
    }

    pub fn symlink_metadata(&self) -> crate::io::Result<Metadata> {
        symlink_metadata(self)
    }

    pub fn canonicalize(&self) -> crate::io::Result<PathBuf> {
        canonicalize(self)
    }

    // pub fn read_link(&self) -> crate::io::Result<PathBuf> {
    //     read_link(self)
    // }

    pub fn read_dir(&self) -> crate::io::Result<ReadDir> {
        read_dir(self)
    }

    pub fn exists(&self) -> bool {
        exists_infallible(self)
    }

    pub fn try_exists(&self) -> crate::io::Result<bool> {
        exists(self)
    }

    pub fn is_file(&self) -> bool {
        self.metadata().is_ok_and(|m| m.is_file())
    }

    pub fn is_dir(&self) -> bool {
        self.metadata().is_ok_and(|m| m.is_dir())
    }

    pub fn is_symlink(&self) -> bool {
        self.symlink_metadata().is_ok()
    }

    pub fn into_path_buf(self: Box<Path>) -> PathBuf {
        let b = Box::into_raw(self);
        unsafe { PathBuf { data: Box::from_raw(b as *mut OsStr).into_os_string() } }
    }
    
    fn from_os_str(os_str: &OsStr) -> &Path {
        // SAFETY: This is just a OsStr anyway!
        unsafe { &*(os_str as *const OsStr as *const Path) }
    }

    fn remove_extraneous_suffixes(&self) -> &Path {
        let without_root = self.as_relative_to_root();
        let mut i = without_root.components();

        loop {
            let Some(a) = i.next_back() else {
                // SAFETY: Part of the string
                return unsafe { self.trim_to_end_of(without_root.as_os_str().as_str()[0..].as_ref()) };
            };

            // SAFETY: Part of the string
            return unsafe { self.trim_to_end_of(a.as_ref()) };
        }
    }

    fn as_relative_to_root(&self) -> &Path {
        let as_str = self.inner.as_str();
        let mut as_str_chars = as_str.chars();

        let Some(first_char) = as_str_chars.next() else {
            // empty?
            return self
        };

        if is_separator(first_char) {
            if self.is_absolute() {
                let a = &as_str[2..];
                let (index, _) = a.char_indices().find(|i| is_separator(i.1)).unwrap();
                a[index + 1..].as_ref()
            }
            else {
                AsRef::<Path>::as_ref(&as_str[1..]).as_relative_to_root()
            }
        }

        else if self.has_drive_letter() {
            if self.is_absolute() {
                // C:\...
                as_str[3..].as_ref()
            }
            else {
                // C:...
                as_str[2..].as_ref()
            }
        }

        else if self.is_relative() {
            self
        }

        else {
            unreachable!()
        }
    }

    // SAFETY: `path` must be part of `self`
    unsafe fn trim_to_end_of(&self, path: &Path) -> &Path {
        let end = path.as_os_str().as_encoded_bytes().as_ptr_range().end;
        let start = self.as_os_str().as_encoded_bytes().as_ptr_range().start;
        let actual_length = (end as usize) - (start as usize);

        // SAFETY: These are part of the same string
        unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(start, actual_length)).as_ref() }
    }

    unsafe fn trim_to_start_of(&self, path: &Path) -> &Path {
        let end = path.as_os_str().as_encoded_bytes().as_ptr_range().start;
        let start = self.as_os_str().as_encoded_bytes().as_ptr_range().start;
        let actual_length = (end as usize) - (start as usize);

        // SAFETY: These are part of the same string
        unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(start, actual_length)).as_ref() }
    }

    unsafe fn rtrim_to_start_of(&self, path: &Path) -> &Path {
        let start = path.as_os_str().as_encoded_bytes().as_ptr_range().start;
        let end = self.as_os_str().as_encoded_bytes().as_ptr_range().end;
        let actual_length = (end as usize) - (start as usize);

        // SAFETY: These are part of the same string
        unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(start, actual_length)).as_ref() }
    }

    pub(crate) fn encode_for_win32(&self) -> Vec<u16> {
        // Some Windows functions violently explode if the path is just "." or ".."
        match self.as_os_str().as_str() {
            "." => return current_dir().unwrap().encode_for_win32(),
            ".." => return current_dir().unwrap().parent().and_then(|p| Some(p.encode_for_win32())).unwrap_or_else(|| vec!['.' as u16, '.' as u16, 0]),
            _ => ()
        }

        let s = self.inner.as_str();
        let mut c = Vec::with_capacity(self.inner.len() * 2 + 1);

        for i in self.components() {
            if !c.is_empty() {
                c.extend_from_slice(MAIN_SEPARATOR.encode_utf16(&mut [0,0]));
            }
            c.extend(i.as_str().encode_utf16());
        }
        c.push(0);
        c.shrink_to_fit();
        c
    }

    pub(crate) fn path_len(&self) -> usize {
        self.as_os_str().as_str().len()
    }
}

impl From<String> for PathBuf {
    fn from(value: String) -> Self {
        PathBuf::from_string(value)
    }
}

impl Deref for PathBuf {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        Path::from_os_str(&self.data)
    }
}

impl Borrow<Path> for PathBuf {
    fn borrow(&self) -> &Path {
        Path::from_os_str(&self.data)
    }
}

impl ToOwned for Path {
    type Owned = PathBuf;
    fn to_owned(&self) -> Self::Owned {
        PathBuf { data: self.inner.to_os_string() }
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<Path> for OsStr {
    fn as_ref(&self) -> &Path {
        Path::from_os_str(self)
    }
}

impl AsRef<Path> for OsString {
    fn as_ref(&self) -> &Path {
        Path::from_os_str(self)
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<Path> for String {
    fn as_ref(&self) -> &Path {
        Path::from_os_str(self.as_ref())
    }
}

impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::from_os_str(self.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct StripPrefixError {

}

pub struct Iter<'a> {
    iterated_amount: usize,
    original_string: &'a str,
    iter: core::str::Split<'a, fn(char) -> bool>
}

impl<'a> Iter<'a> {
    pub fn as_path(&self) -> &Path {
        self.original_string[self.iterated_amount..].as_ref()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<Self::Item> {
        let string = self.iter.next().map(OsStr::from_str)?;
        self.iterated_amount += string.len() + 1;
        Some(string)
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(OsStr::from_str)
    }
}

impl<'a> IntoIterator for &'a Path {
    type Item = &'a OsStr;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a PathBuf {
    type Item = &'a OsStr;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Components<'a> {
    iter: Iter<'a>
}

impl<'a> Iterator for Components<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let i = self.iter.next()?;
            if i.as_str() == "" || i.as_str() == "." {
                continue
            }
            return Some(i)
        }
    }
}

impl<'a> DoubleEndedIterator for Components<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let i = self.iter.next_back()?;
            if i.as_str() == "" || i.as_str() == "." {
                continue
            }
            return Some(i)
        }
    }
}

pub struct Ancestors<'a> {
    path: &'a Path
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = &'a Path;
    fn next(&mut self) -> Option<Self::Item> {
        self.path = self.path.parent()?;
        Some(self.path)
    }
}

impl From<&Path> for Arc<Path> {
    fn from(value: &Path) -> Self {
        value.to_path_buf().into_boxed_path().into()
    }
}

impl From<&Path> for Box<Path> {
    fn from(value: &Path) -> Self {
        value.to_path_buf().into_boxed_path()
    }
}

impl From<&Path> for Rc<Path> {
    fn from(value: &Path) -> Self {
        value.to_path_buf().into_boxed_path().into()
    }
}

impl<'a> From<&'a Path> for Cow<'a, Path> {
    fn from(value: &'a Path) -> Self {
        Cow::Borrowed(value)
    }
}

impl From<&mut Path> for Arc<Path> {
    fn from(value: &mut Path) -> Self {
        Arc::from(&*value)
    }
}

impl From<&mut Path> for Box<Path> {
    fn from(value: &mut Path) -> Self {
        Box::from(&*value)
    }
}

impl From<&mut Path> for Rc<Path> {
    fn from(value: &mut Path) -> Self {
        Rc::from(&*value)
    }
}

impl AsRef<OsStr> for Path {
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl From<Cow<'_, Path>> for Box<Path> {
    fn from(value: Cow<'_, Path>) -> Self {
        match value {
            Cow::Borrowed(p) => p.to_path_buf().into_boxed_path(),
            Cow::Owned(p) => p.into_boxed_path()
        }
    }
}

impl Clone for Box<Path> {
    fn clone(&self) -> Self {
        self.to_path_buf().into_boxed_path()
    }
}

impl AsRef<Path> for Components<'_> {
    fn as_ref(&self) -> &Path {
        self.iter.as_path()
    }
}

impl From<PathBuf> for Box<Path> {
    fn from(value: PathBuf) -> Self {
        value.into_boxed_path()
    }
}

#[cfg(test)]
mod test;
