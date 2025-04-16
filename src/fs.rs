use alloc::string::String;
pub(crate) const MAX_PATH: usize = windows_sys::Win32::Foundation::MAX_PATH as usize;

#[derive(Clone, PartialEq, Debug)]
pub struct PathBuf {
    data: String
}

impl PathBuf {
    pub(crate) fn from_string(data: String) -> Self {
        Self { data }
    }
}


