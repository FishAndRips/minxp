use crate::path::Path;

pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> crate::io::Result<u64> {
    todo!()
}

pub fn create_dir<P: AsRef<Path>>(path: P) -> crate::io::Result<()> {
    todo!()
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> crate::io::Result<()> {
    todo!()
}

pub struct ReadDir;
pub fn read_dir<P: AsRef<Path>>(path: P) -> crate::io::Result<ReadDir> {
    todo!()
}

pub fn remove_dir<P: AsRef<Path>>(path: P) -> crate::io::Result<()> {
    todo!()
}

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> crate::io::Result<()> {
    todo!()
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> crate::io::Result<()> {
    todo!()
}
