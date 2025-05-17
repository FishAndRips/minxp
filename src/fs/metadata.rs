use crate::util::get_proc_from_module;
use crate::io::Error;
use crate::path::{Path, PathBuf, MAX_PATH_EXTENDED};
use crate::util::get_last_windows_error;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::{format, vec};
use core::mem::zeroed;
use core::ptr::{null, null_mut};
use spin::Lazy;
use windows_sys::Win32::Foundation::{CloseHandle, ERROR_FILE_NOT_FOUND, FALSE, FILETIME, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileW, GetFileInformationByHandle, GetFullPathNameW, BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_NAME_NORMALIZED, GETFINALPATHNAMEBYHANDLE_FLAGS, OPEN_EXISTING};
use windows_sys::Win32::UI::Shell::PathFileExistsW;

#[derive(Clone)]
pub struct Metadata {
    attribute_data: BY_HANDLE_FILE_INFORMATION
}

pub struct FileType {
    data: u32
}

#[derive(Clone, Debug, PartialEq)]
pub struct Permissions {
    read_only: bool
}

impl Permissions {
    pub fn readonly(&self) -> bool {
        self.read_only
    }
    pub fn set_readonly(&mut self, read_only: bool) {
        self.read_only = read_only
    }
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        (self.data & FILE_ATTRIBUTE_DIRECTORY) != 0
    }
    pub fn is_file(&self) -> bool {
        !self.is_dir()
    }
    pub fn is_symlink(&self) -> bool {
        (self.data & FILE_ATTRIBUTE_REPARSE_POINT) != 0
    }
}

impl Metadata {
    pub(crate) fn new(attribute_data: BY_HANDLE_FILE_INFORMATION) -> Self {
        Self { attribute_data }
    }

    pub fn file_type(&self) -> FileType {
        FileType { data: self.attribute_data.dwFileAttributes }
    }

    pub fn is_dir(&self) -> bool {
        self.file_type().is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.file_type().is_file()
    }

    pub fn is_symlink(&self) -> bool {
        self.file_type().is_symlink()
    }

    pub fn len(&self) -> u64 {
        let low = self.attribute_data.nFileSizeLow as u64;
        let high = self.attribute_data.nFileSizeHigh as u64;
        high << 32 | low
    }

    pub fn permission(&self) -> Permissions {
        Permissions { read_only: (self.attribute_data.dwFileAttributes & FILE_ATTRIBUTE_READONLY) != 0 }
    }

    // TODO: Convert from 100-nanosecond precision to some sort of SystemTime
    //
    // This should return a SystemTime, but we don't have it implemented yet.
    #[deprecated = "this will be changed to a SystemTime, do NOT use this right now if you don't want your code to break on a future version"]
    pub fn modified(&self) -> FILETIME {
        self.attribute_data.ftLastWriteTime
    }

    // TODO: Convert from 100-nanosecond precision to some sort of SystemTime
    //
    // This should return a SystemTime, but we don't have it implemented yet.
    #[deprecated = "this will be changed to a SystemTime, do NOT use this right now if you don't want your code to break on a future version"]
    pub fn accessed(&self) -> FILETIME {
        self.attribute_data.ftLastAccessTime
    }

    // TODO: Convert from 100-nanosecond precision to some sort of SystemTime
    //
    // This should return a SystemTime, but we don't have it implemented yet.
    #[deprecated = "this will be changed to a SystemTime, do NOT use this right now if you don't want your code to break on a future version"]
    pub fn created(&self) -> FILETIME {
        self.attribute_data.ftCreationTime
    }
}

pub fn metadata<P: AsRef<Path>>(path: P) -> crate::io::Result<Metadata> {
    metadata_impl(path, false)
}

pub fn symlink_metadata<P: AsRef<Path>>(path: P) -> crate::io::Result<Metadata> {
    metadata_impl(path, true)
}

fn metadata_impl<P: AsRef<Path>>(path: P, follow_symlink: bool) -> crate::io::Result<Metadata> {
    // SAFETY: Safe to be zeroed out
    let mut information = unsafe { zeroed() };

    let file = open_file_for_querying_metadata(path, follow_symlink)?;

    let success = unsafe {
        GetFileInformationByHandle(
            file,
            &mut information
        )
    };
    let error = get_last_windows_error();

    // Close the file
    unsafe { CloseHandle(file) };

    if success == FALSE {
        return Err(Error { reason: format!("unable to get file metadata: {error}") })
    }

    Ok(Metadata::new(information))
}

pub(crate) fn open_file_for_querying_metadata<P: AsRef<Path>>(path: P, follow_symlink: bool) -> crate::io::Result<HANDLE> {
    let result = unsafe {
        CreateFileW(
            path.as_ref().encode_for_win32().as_ptr(),
            0,
            0,
            null(),
            OPEN_EXISTING,
            if follow_symlink { FILE_FLAG_OPEN_REPARSE_POINT } else { 0 } | FILE_FLAG_BACKUP_SEMANTICS,
            null_mut()
        )
    };
    let error = get_last_windows_error();
    if result == INVALID_HANDLE_VALUE {
        return Err(Error { reason: format!("failed to open file for metadata: {error}") })
    }
    Ok(result)
}

/// Checks if the file exists.
///
/// This function returns `Ok(true)` if it does, `Ok(false)` if it doesn't, and `Err()` if an error
/// occurred.
pub fn exists<P: AsRef<Path>>(path: P) -> crate::io::Result<bool> {
    if exists_infallible(path) {
        Ok(true)
    }
    else {
        let error = get_last_windows_error();
        if error == ERROR_FILE_NOT_FOUND {
            Ok(false)
        }
        else {
            Err(Error { reason: format!("unable to check if a file exists: {error}") })
        }
    }
}

pub(crate) fn exists_infallible<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref().encode_for_win32();
    unsafe { PathFileExistsW(path.as_ptr()) != FALSE }
}

/// Resolves a path to its actual, absolute path, with all symlinks resolved.
///
/// # Compatibility notes
///
/// - On Windows Vista or newer, this uses the [`GetFinalPathNameByHandleW`] function, which also
///   resolves symlinks.
/// - On Windows XP and older, this will just return the absolute path with an additional check for
///   if the path exists.
///
/// [`GetFinalPathNameByHandleW`]: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew
pub fn canonicalize<P: AsRef<Path>>(path: P) -> crate::io::Result<PathBuf> {
    CANONICALIZE(path.as_ref())
}

/// Convert the path to its absolute form.
///
/// If the path is already absolute, this simply returns the path as a PathBuf. Otherwise, this uses
/// the `GetFullPathNameW` function to get a full path.
///
/// [`GetFullPathNameW`]: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfullpathnamew
pub fn absolute<P: AsRef<Path>>(path: P) -> crate::io::Result<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
        return Ok(path.to_owned());
    }

    let mut full_data = vec![0u16; MAX_PATH_EXTENDED + 1];
    let len = unsafe {
        GetFullPathNameW(
            path.encode_for_win32().as_ptr(),
            full_data.len() as u32,
            full_data.as_mut_ptr(),
            null_mut()
        )
    } as usize;

    if len == 0 || len > full_data.len() {
        let error = get_last_windows_error();
        return Err(Error { reason: format!("failed to get absolute path: {error}")})
    }

    full_data.truncate(len);
    Ok(String::from_utf16(full_data.as_slice()).expect("GetFullPathNameW did not return a UTF-16 path").into())
}

type GetFinalPathNameByHandleW = unsafe extern "system" fn (HANDLE, windows_sys::core::PWSTR, u32, GETFINALPATHNAMEBYHANDLE_FLAGS) -> u32;

static CANONICALIZE: Lazy<Box<dyn Fn(&Path) -> crate::io::Result<PathBuf> + Send + Sync>> = Lazy::new(|| {
    let get_final_path_name_by_handle: Option<GetFinalPathNameByHandleW> = get_proc_from_module!(
        "kernel32.dll",
        "GetFinalPathNameByHandleW"
    );

    if let Some(f) = get_final_path_name_by_handle {
        Box::new(move |p| resolve_path_modern(p, f))
    }
    else {
        Box::new(|p| resolve_path_fallback(p))
    }
});

fn resolve_path_modern(
    path: &Path,
    get_final_path_name_by_handle_w: GetFinalPathNameByHandleW
) -> crate::io::Result<PathBuf> {
    let file = open_file_for_querying_metadata(path, false)?;
    let mut full_data = vec![0u16; MAX_PATH_EXTENDED + 1];
    let q = unsafe {
        get_final_path_name_by_handle_w(file, full_data.as_mut_ptr(), full_data.len() as u32, FILE_NAME_NORMALIZED)
    };
    let error = get_last_windows_error();
    unsafe { CloseHandle(file) };

    if q == 0 {
        return Err(Error { reason: format!("failed to get final path name: {error}") })
    }

    full_data.truncate(q as usize);

    Ok(String::from_utf16(full_data.as_slice()).expect("GetFinalPathNameByHandleW did not return UTF-16").into())
}

fn resolve_path_fallback(path: &Path) -> crate::io::Result<PathBuf> {
    // Check if it exists. If so, get the absolute path
    let path = path.as_ref();
    if !path.exists() {
        return Err(Error { reason: "path does not exists".to_owned() })
    }
    absolute(path)
}
