use crate::ffi::OsString;
use crate::fs::Metadata;
use crate::io::Error;
use crate::io::Result;
use crate::path::{Path, PathBuf, MAIN_SEPARATOR};
use crate::util::get_last_windows_error;
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::zeroed;
use core::ptr::{null, null_mut};
use spin::Mutex;
use windows_sys::Win32::Foundation::{FALSE, HANDLE};
use windows_sys::Win32::Storage::FileSystem::{CreateDirectoryW, FindClose, FindFirstFileW, FindNextFileW, RemoveDirectoryW, WIN32_FIND_DATAW};
use windows_sys::Win32::UI::Shell::{SHFileOperationW, FOF_NOCONFIRMATION, FOF_NOCONFIRMMKDIR, FOF_NOERRORUI, FOF_SILENT, FO_DELETE, SHFILEOPSTRUCTW};

pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let success = unsafe {
        CreateDirectoryW(
            path.as_ref().encode_for_win32().as_ptr(),
            null()
        )
    };

    if success == FALSE {
        let error = get_last_windows_error();
        return Err(Error { reason: format!("failed to create directory: {error}") })
    }

    Ok(())
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    let all_ancestors: Vec<&Path> = path.as_ref().ancestors().collect();
    for i in all_ancestors.into_iter().rev() {
        if i.path_len() == 0 {
            continue
        }
        if !i.exists() {
            create_dir(i)?;
        }
    }

    Ok(())
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<ReadDir> {
    let mut find_data: WIN32_FIND_DATAW = unsafe { zeroed() };
    let base_path = Arc::new(path.as_ref().to_owned());

    let canonicalized_path = base_path.canonicalize()?;
    let mut path_utf16: Vec<u16> = canonicalized_path.encode_for_win32();
    path_utf16.pop();
    path_utf16.push(MAIN_SEPARATOR as u16);
    path_utf16.push('*' as u16);
    path_utf16.push(0);

    let handle = unsafe { FindFirstFileW(path_utf16.as_ptr(), &mut find_data) };
    if handle.is_null() {
        let error = get_last_windows_error();
        return Err(Error { reason: format!("cannot traverse directory: {error}") })
    }

    Ok(ReadDir {
        find_data,
        handle,
        base_path
    })
}

pub fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let success = unsafe {
        RemoveDirectoryW(
            path.as_ref().encode_for_win32().as_ptr()
        )
    };

    if success == FALSE {
        let error = get_last_windows_error();
        return Err(Error { reason: format!("failed to delete directory: {error}") })
    }

    Ok(())
}

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    if !path.as_ref().is_dir() {
        return Err(Error { reason: "cannot remove all dir: not a directory".to_string() })
    }

    let mut op: SHFILEOPSTRUCTW = unsafe { zeroed() };
    let mut path_data = path.as_ref().encode_for_win32();
    let mut path_data_2 = path_data.clone();

    // Remove the null terminator, and add `\*` to the end to remove everything inside the dir.
    path_data.pop();
    path_data.push(MAIN_SEPARATOR as u16);
    path_data.push('*' as u16);
    path_data.push(0);

    // Also delete the directory itself.
    path_data.append(&mut path_data_2);
    path_data.push(0);

    // No more files to remove.
    path_data.push(0);

    op.pFrom = path_data.as_ptr();
    op.wFunc = FO_DELETE;
    op.fFlags = (FOF_SILENT | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_NOCONFIRMMKDIR) as u16;

    let success = unsafe {
        SHFileOperationW(
            &mut op
        )
    };

    // returns nonzero on fail
    if success != 0 {
        return Err(Error { reason: format!("failed to remove all dir: {success}") })
    }

    Ok(())
}

pub struct ReadDir {
    find_data: WIN32_FIND_DATAW,
    handle: HANDLE,
    base_path: Arc<PathBuf>
}

impl Iterator for ReadDir {
    type Item = Result<DirEntry>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.handle.is_null() {
            return None
        }

        let mut file_name = String::from_utf16(&self.find_data.cFileName)
            .expect("got a non-utf-16 path when traversing a directory");

        if let Some(f) = file_name.find(0 as char) {
            file_name.truncate(f);
        }

        let succeed = unsafe { FindNextFileW(self.handle, &mut self.find_data) } != FALSE;
        if !succeed {
            unsafe { FindClose(self.handle) };
            self.handle = null_mut();
        }

        if file_name == "." || file_name == ".." || file_name == "" {
            return self.next()
        }

        Some(Ok(DirEntry {
            base_path: self.base_path.clone(),
            filename: file_name.into(),
            joined: Mutex::default()
        }))
    }
}

impl Drop for ReadDir {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { FindClose(self.handle) };
        }
    }
}

pub struct DirEntry {
    base_path: Arc<PathBuf>,
    filename: OsString,
    joined: Mutex<Option<Arc<PathBuf>>>
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        PathBuf::clone(&self.get_path())
    }
    pub fn metadata(&self) -> Result<Metadata> {
        self.get_path().metadata()
    }

    fn get_path(&self) -> Arc<PathBuf> {
        let mut a = self.joined.lock();

        match a.as_ref() {
            Some(k) => Arc::clone(k),
            None => {
                let path = Arc::new(self.base_path.join(&self.filename));
                *a = Some(path.clone());
                path
            }
        }
    }
}
