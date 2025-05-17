use alloc::string::String;
use core::fmt::{Display, Formatter};
use core::marker::PhantomData;
use crate::ffi::{OsStr, OsString};
use crate::path::PathBuf;

#[derive(Debug)]
pub struct JoinPathsError {
    phantom_data: PhantomData<()>
}

impl Display for JoinPathsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("an error occurred when joining paths")
    }
}

pub fn join_paths<I: Iterator<Item = T>, T: AsRef<OsStr>>(paths: I) -> Result<OsString, JoinPathsError> {
    let mut s = String::new();
    for i in paths {
        let path_str = i.as_ref().as_str();

        if path_str.contains('"') {
            return Err(JoinPathsError { phantom_data: PhantomData });
        }

        if s.is_empty() {
            s += ";"
        }

        if path_str.contains(';') {
            s += "\"";
            s += path_str;
            s += "\"";
        }
        else {
            s += path_str;
        }
    }

    Ok(s.into())
}

pub fn split_paths<T: AsRef<OsStr> + ?Sized>(unparsed: &T) -> SplitPaths<'_> {
    SplitPaths {
        path_data: Some(unparsed.as_ref().as_str())
    }
}

pub struct SplitPaths<'a> {
    path_data: Option<&'a str>
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        let string = self.path_data?;

        let string_found;
        let mut current_offset = 0usize;

        loop {
            let search_string = &string[current_offset..];

            let Some(next_semicolon) = search_string.find(';') else {
                // last string
                string_found = string;
                self.path_data = None;
                break;
            };

            match search_string.find('"') {
                Some(next_quote) if next_quote < next_semicolon => {
                    current_offset += next_quote + 1;
                    let remaining = &string[current_offset..];
                    let Some(end) = remaining.find('"') else {
                        // no quote to terminate, so last string
                        string_found = string;
                        self.path_data = None;
                        break;
                    };
                    current_offset += end + 1;
                },
                _ => {
                    let end = current_offset + next_semicolon;
                    let (a, b) = string.split_at(end);
                    string_found = a;
                    self.path_data = Some(&b[1..]);
                    break;
                }
            }
        }

        let found_string = string_found.replace('"', "");
        Some(PathBuf::from_string(found_string))
    }
}

#[cfg(test)]
mod test;
