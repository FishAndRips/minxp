use crate::fs::Metadata;
use crate::io::*;
use crate::path::Path;
use crate::util::get_last_windows_error;
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::zeroed;
use core::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{CloseHandle, FALSE, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CopyFileExW, CreateFileW, DeleteFileW, FlushFileBuffers, GetFileInformationByHandle, ReadFile, SetFilePointerEx, WriteFile, CREATE_ALWAYS, CREATE_NEW, FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL, FILE_BEGIN, FILE_CURRENT, FILE_END, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, TRUNCATE_EXISTING};

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    File::create(path).and_then(|mut f| f.write_all(contents.as_ref()))
}

pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut v = Vec::new();
    file.read_to_end(&mut v)?;
    Ok(v)
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    read(path).and_then(|s| {
        String::from_utf8(s).map_err(|e| Error { reason: format!("cannot read_to_string due to UTF-8 parsing error: {e:?}") })
    })
}

pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64> {
    let success = unsafe {
        CopyFileExW(
            from.as_ref().encode_for_win32().as_ptr(),
            to.as_ref().encode_for_win32().as_ptr(),
            None,
            null(),
            null_mut(),
            0
        )
    };

    if success == FALSE {
        let error = get_last_windows_error();
        return Err(Error { reason: format!("failed to copy: {error}") })
    }

    Ok(to.as_ref().metadata()?.len())
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let success = unsafe {
        DeleteFileW(
            path.as_ref().encode_for_win32().as_ptr()
        )
    };

    if success == FALSE {
        let error = get_last_windows_error();
        return Err(Error { reason: format!("failed to delete file: {error}") })
    }

    Ok(())
}

pub struct File {
    handle: HANDLE
}

impl File {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::options().read(true).open(path)
    }

    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::options().read(true).write(true).create(true).open(path)
    }

    pub fn create_new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::options().read(true).write(true).create_new(true).open(path)
    }

    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub fn metadata(&self) -> Result<Metadata> {
        let mut file_info = unsafe { zeroed() };
        let result = unsafe { GetFileInformationByHandle(self.handle, &mut file_info) };
        if result == FALSE {
            let error = get_last_windows_error();
            return Err(Error { reason: format!("failed to get metadata for an open file: {error}") })
        }
        Ok(Metadata::new(file_info))
    }

    fn remaining_data_in_stream(&mut self) -> Result<u64> {
        let pos = self.seek_position()?;
        let end = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(pos))?;
        Ok(end - pos)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes_to_read = buf.len().min(u32::MAX as usize) as u32;
        let mut bytes_read = 0;

        let success = unsafe { ReadFile(
            self.handle,
            buf.as_mut_ptr(),
            bytes_to_read,
            &mut bytes_read,
            null_mut()
        ) };

        if success == FALSE {
            let reason = get_last_windows_error();
            return Err(Error { reason: format!("failed to read file: {reason}") })
        }

        Ok(bytes_read as usize)
    }

    fn read_to_end(&mut self, data: &mut Vec<u8>) -> Result<()> {
        let len = self.remaining_data_in_stream()?;
        if len > (isize::MAX as u64) || data.try_reserve_exact(len as usize).is_err() {
            return Err(Error { reason: "not enough RAM to read the file".to_owned() })
        }

        while data.len() < len as usize {
            let start = data.len();
            data.resize(data.capacity(), 0);
            match self.read(&mut data[start..]) {
                Ok(d) => data.truncate(start + d),
                Err(e) => {
                    core::mem::take(data);
                    return Err(e)
                }
            }

            // TODO: Better handle EOF?
            if start == data.len() {
                return Ok(());
            }
        }

        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if self.read(buf)? != buf.len() {
            // ErrorKind::UnexpectedEof
            Err(Error { reason: "file smaller than buffer".to_owned() })
        }
        else {
            Ok(())
        }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let bytes_to_write = buf.len().min(u32::MAX as usize) as u32;
        let mut number_of_bytes_written = 0;
        let result = unsafe {
            WriteFile(
                self.handle,
                buf.as_ptr(),
                bytes_to_write,
                &mut number_of_bytes_written,
                null_mut()
            )
        };
        if result == FALSE {
            let error = get_last_windows_error();
            return Err(Error { reason: format!("failed to write to file: {error}") })
        }
        Ok(number_of_bytes_written as usize)
    }

    fn flush(&mut self) -> Result<()> {
        let result = unsafe {
            FlushFileBuffers(self.handle)
        };
        if result == FALSE {
            let error = get_last_windows_error();
            return Err(Error { reason: format!("failed to flush file: {error}") })
        }
        Ok(())
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle) };
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let (offset, method) = match pos {
            SeekFrom::Current(q) => (q, FILE_CURRENT),
            SeekFrom::Start(q) => (q as i64, FILE_BEGIN), // will be interpreted as unsigned by SetFilePointerEx
            SeekFrom::End(q) => (q, FILE_END),
        };

        let mut new_offset = 0u64;

        let success = unsafe {
            SetFilePointerEx(
                self.handle,
                offset as i64,
                &mut new_offset as *mut _ as *mut i64,
                method
            )
        };

        if success == FALSE {
            let last_error = get_last_windows_error();
            return Err(Error { reason: format!("failed to seek: {last_error}") })
        };

        Ok(new_offset)
    }
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}
impl OpenOptions {
    pub fn new() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        let is_writable = self.write || self.append;
        if !is_writable && self.create {
            return Err(Error { reason: format!("open with create but not write or append") });
        }
        if !is_writable && self.create_new {
            return Err(Error { reason: format!("open with create_new but not write or append") });
        }

        let mut desired_access = 0;
        let mut share_access = 0;

        if self.write {
            desired_access |= GENERIC_WRITE;
            share_access |= FILE_SHARE_WRITE;
        }
        if self.read {
            desired_access |= GENERIC_READ;
            share_access |= FILE_SHARE_READ;
        }
        if self.append {
            desired_access |= FILE_APPEND_DATA;
            share_access |= FILE_SHARE_WRITE;
        }

        let handle = unsafe {
            CreateFileW(
                path.as_ref().encode_for_win32().as_ptr(),
                desired_access,
                share_access,
                null(),
                if self.create_new {
                    CREATE_NEW
                }
                else if self.create {
                    CREATE_ALWAYS
                }
                else if self.truncate {
                    TRUNCATE_EXISTING
                }
                else {
                    OPEN_EXISTING
                },
                FILE_ATTRIBUTE_NORMAL,
                null_mut()
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            let error = get_last_windows_error();
            return Err(Error { reason: format!("cannot open file: {error}") })
        }

        Ok(File { handle })
    }
}
