use super::*;

fn same_path<A: AsRef<Path>, B: AsRef<Path>>(a: A, b: B) {
    assert_eq!(a.as_ref().as_os_str().as_str(), b.as_ref().as_os_str().as_str());
}

#[test]
fn remove_extraneous_suffixes() {
    same_path(Path::new("C:\\Users\\Something").remove_extraneous_suffixes(), "C:\\Users\\Something");
    same_path(Path::new("C:\\Users\\Something\\").remove_extraneous_suffixes(), "C:\\Users\\Something");
    same_path(Path::new("C:\\Users\\Something\\.\\.\\..\\.\\.\\./\\\\").remove_extraneous_suffixes(), "C:\\Users\\Something\\.\\.\\..");
}

#[test]
fn relative_to_root() {
    same_path(Path::new("C:\\Users\\Something\\").as_relative_to_root(), "Users\\Something\\");
    same_path(Path::new("\\\\Drive\\Something\\").as_relative_to_root(), "Something\\");
    same_path(Path::new("\\\\\\").as_relative_to_root(), "");
}

#[test]
fn as_path() {
    same_path(Path::new("C:\\Users\\Something\\").to_owned().as_path(), "C:\\Users\\Something\\");
}

#[test]
fn push() {
    same_path({
                  let mut a = Path::new("C:\\Users\\Something").to_owned();
                  a.push("OrRather");
                  a
              }, "C:\\Users\\Something\\OrRather");
    same_path({
                  let mut a = Path::new("C:\\Users\\Something\\").to_owned();
                  a.push("OrRather");
                  a
              }, "C:\\Users\\Something\\OrRather");
}

#[test]
fn pop() {
    same_path({
                  let mut a = Path::new("C:\\Users\\Something\\").to_owned();
                  a.pop();
                  a
              }, "C:\\Users");
    same_path({
                  let mut a = Path::new("C:\\").to_owned();
                  a.pop();
                  a
              }, "C:\\");
}

#[test]
fn set_file_name() {
    same_path({
                  let mut a = Path::new("C:\\Users\\Something").to_owned();
                  a.set_file_name("Red");
                  a
              }, "C:\\Users\\Red");
    same_path({
                  let mut a = Path::new("C:\\Users\\Blue.txt").to_owned();
                  a.set_file_name(".txt");
                  a
              }, "C:\\Users\\.txt");
    same_path({
                  let mut a = Path::new("C:\\").to_owned();
                  a.set_file_name("jxl.jxl");
                  a
              }, "C:\\jxl.jxl");
}

#[test]
fn set_extension() {
    same_path({
                  let mut a = Path::new("C:\\Users\\Something").to_owned();
                  a.set_extension("txt");
                  a
              }, "C:\\Users\\Something.txt");
    same_path({
                  let mut a = Path::new("C:\\Users\\Something").to_owned();
                  a.set_extension(".txt");
                  a
              }, "C:\\Users\\Something..txt");
    same_path({
                  let mut a = Path::new("C:\\Users\\Something.txt").to_owned();
                  a.set_extension("jxl");
                  a
              }, "C:\\Users\\Something.jxl");
}

#[test]
fn truncate_extraneous_suffixes() {
    same_path({
                  let mut a = Path::new("C:\\Users\\Something").to_owned();
                  a.truncate_extraneous_suffixes();
                  a
              }, "C:\\Users\\Something");
    same_path({
                  let mut a = Path::new("C:\\Users\\Something\\").to_owned();
                  a.truncate_extraneous_suffixes();
                  a
              }, "C:\\Users\\Something");
    same_path({
                  let mut a = Path::new("C:\\Users\\Something\\.\\.\\..\\.\\.\\.\\\\\\").to_owned();
                  a.truncate_extraneous_suffixes();
                  a
              }, "C:\\Users\\Something\\.\\.\\..");
}

#[test]
fn as_mut_os_string() {
    same_path(Path::new("C:\\Users\\Something").to_owned().as_mut_os_string(), "C:\\Users\\Something");
}

#[test]
fn into_os_string() {
    same_path(Path::new("C:\\Users\\Something").to_owned().into_os_string(), "C:\\Users\\Something");
}

#[test]
fn into_boxed_path() {
    same_path(Path::new("C:\\Users\\Something").to_owned().into_boxed_path(), "C:\\Users\\Something");
}

#[test]
fn as_os_str() {
    assert_eq!(Path::new("some sort of string").as_os_str().as_str(), "some sort of string");
}

#[test]
fn to_str() {
    assert_eq!(Path::new("some sort of string").to_str().unwrap(), "some sort of string");
}

#[test]
fn to_string_lossy() {
    assert_eq!(Path::new("some sort of string").to_string_lossy(), "some sort of string");
}

#[test]
fn to_path_buf() {
    assert_eq!(Path::new("some sort of string").to_path_buf().as_os_str().as_str(), "some sort of string");
}

#[test]
fn is_absolute() {
    assert!(!Path::new("C:something").is_absolute());
    assert!(!Path::new("c:something").is_absolute());
    assert!(Path::new("C:\\something").is_absolute());
    assert!(Path::new("c:\\something").is_absolute());
    assert!(Path::new("C:/something").is_absolute());
    assert!(Path::new("c:/something").is_absolute());
    assert!(!Path::new("\\something").is_absolute());
    assert!(!Path::new("\\\\something").is_absolute());
    assert!(Path::new("\\\\something\\").is_absolute());
    assert!(!Path::new("CCCCC:\\something").is_absolute());
}

#[test]
fn has_drive_letter() {
    assert!(Path::new("C:something").has_drive_letter());
    assert!(Path::new("c:something").has_drive_letter());
    assert!(Path::new("C:\\something").has_drive_letter());
    assert!(Path::new("c:\\something").has_drive_letter());
    assert!(Path::new("C:/something").has_drive_letter());
    assert!(Path::new("c:/something").has_drive_letter());
    assert!(!Path::new("\\something").has_drive_letter());
    assert!(!Path::new("\\\\something").has_drive_letter());
    assert!(!Path::new("CCCCC:\\something").has_drive_letter());
}

#[test]
fn is_relative() {
    assert!(Path::new("C:something").is_relative());
    assert!(Path::new("c:something").is_relative());
    assert!(!Path::new("C:\\something").is_relative());
    assert!(!Path::new("c:\\something").is_relative());
    assert!(!Path::new("C:/something").is_relative());
    assert!(!Path::new("c:/something").is_relative());
    assert!(Path::new("\\something").is_relative());
    assert!(Path::new("\\\\something").is_relative());
    assert!(!Path::new("\\\\something\\").is_relative());
    assert!(Path::new("CCCCC:\\something").is_relative());
}

#[test]
fn has_root() {
    assert!(!Path::new("C:something").has_root());
    assert!(!Path::new("c:something").has_root());
    assert!(Path::new("C:\\something").has_root());
    assert!(Path::new("c:\\something").has_root());
    assert!(Path::new("C:/something").has_root());
    assert!(Path::new("c:/something").has_root());
    assert!(Path::new("\\something").has_root());
    assert!(Path::new("\\\\something").has_root());
    assert!(Path::new("\\\\something\\").has_root());
    assert!(!Path::new("CCCCC:\\something").has_root());
}

#[test]
fn utf16() {
    // null bytes are stripped from OsStr::from_str
    assert_eq!(String::from_utf16(&Path::new("C:\\Users\\Something").encode_for_win32()).unwrap(), "C:\\Users\\Something\x00");
    same_path(String::from_utf16(&Path::new("C:\\Users\\Something").encode_for_win32()).unwrap(), "C:\\Users\\Something");
    same_path(String::from_utf16(&Path::new("C:\\Users////\\Something").encode_for_win32()).unwrap(), "C:\\Users\\Something");
}

#[test]
fn parent() {
    same_path(Path::new("C:\\Users\\Something\\").parent().unwrap(), "C:\\Users");
    same_path(Path::new("C:\\Users").parent().unwrap(), "C:\\");
}

#[test]
fn ancestors() {
    let mut ancestors = Path::new("C:\\Users\\Something\\Or/.\\Rather\\\\//////\\././././././././\\.\\Thing/").ancestors();
    same_path(ancestors.next().unwrap(), "C:\\Users\\Something\\Or/.\\Rather");
    same_path(ancestors.next().unwrap(), "C:\\Users\\Something\\Or");
    same_path(ancestors.next().unwrap(), "C:\\Users\\Something");
    same_path(ancestors.next().unwrap(), "C:\\Users");
    same_path(ancestors.next().unwrap(), "C:\\");
    assert!(ancestors.next().is_none());
}

#[test]
fn file_name() {
    assert_eq!(Path::new("C:\\Users\\Something\\Or/.\\Rather\\\\//////\\././././././././\\.\\Thing/").file_name().unwrap(), "Thing");
    assert_eq!(Path::new("C:\\Users\\Something.txt").file_name().unwrap(), "Something.txt");
    assert!(Path::new("C:\\").file_name().is_none());
}

#[test]
fn strip_prefix() {
    assert_eq!(
        Path::new("C:\\Users\\Something\\or").strip_prefix("C:\\Users\\\\Something").unwrap(),
        Path::new("or")
    );
    assert_eq!(
        Path::new("C:\\Users\\\\\\Something\\or\\rather").strip_prefix("C:\\Users\\\\Something\\or").unwrap(),
        Path::new("rather")
    );
    assert!(
        Path::new("C:\\Users\\\\\\Something").strip_prefix("C:\\Users\\\\Somet").is_err()
    );
    assert!(
        Path::new("C:\\Users\\\\\\Something").strip_prefix("D:\\Users\\\\").is_err()
    );
}

#[test]
fn ends_with() {
    assert!(Path::new("C:\\Users\\Something").ends_with(Path::new("C:\\Users\\\\Something")));
    assert!(Path::new("C:\\Users\\Something").ends_with(Path::new("Users/Something")));
    assert!(Path::new("C:\\Users\\Something").ends_with(Path::new("Something")));
    assert!(Path::new("C:\\Users\\Something").ends_with(Path::new("Something///////./")));
    assert!(!Path::new("C:\\Users\\Something").ends_with(Path::new("thing")));
}

#[test]
fn starts_with() {
    assert!(Path::new("C:\\Users\\Something").starts_with(Path::new("C:\\Users\\\\Something")));
    assert!(!Path::new("C:\\Users\\Something").starts_with(Path::new("C:\\Users\\\\Something\\or")));
    assert!(Path::new("C:\\Users\\Something\\or\\").starts_with(Path::new("C:\\Users\\\\Something\\or")));
    assert!(Path::new("C:\\Users\\Something\\or").starts_with(Path::new("C:\\Users\\\\Something\\or\\")));
    assert!(Path::new("C:\\Users\\Something\\or").starts_with(Path::new("C:\\Users\\\\Something\\")));
    assert!(Path::new("C:\\Users\\Something\\or").starts_with(Path::new("C:\\Users\\\\Something")));
    assert!(Path::new("C:\\Users\\Something\\or").starts_with(Path::new("C:\\Users\\\\Something/")));
    assert!(Path::new("C:\\Users\\Something\\or").starts_with(Path::new("C:\\Users\\\\Something/.")));
}

#[test]
fn file_stem() {
    assert_eq!(Path::new("C:\\Users\\Something\\Or/.\\Rather\\\\//////\\././././././././\\.\\Thing/").file_stem().unwrap(), "Thing");
    assert_eq!(Path::new("C:\\Users\\Something.txt").file_stem().unwrap(), "Something");
    assert!(Path::new("C:\\").file_stem().is_none());
}

#[test]
fn extension() {
    assert_eq!(Path::new("C:\\Users\\Something\\Or/.\\Rather\\\\//////\\././././././././\\.\\Thing/").extension(), None);
    assert_eq!(Path::new("C:\\Users\\Something.txt").extension().unwrap().as_str(), "txt");
}

#[test]
fn join() {
    same_path(Path::new("C:\\Users\\Something.txt\\").join("test"), "C:\\Users\\Something.txt\\test");
    same_path(Path::new("C:\\Users\\Something.txt/").join("test"), "C:\\Users\\Something.txt/test");
    same_path(Path::new("C:\\Users\\Something.txt").join("test"), "C:\\Users\\Something.txt\\test");
    same_path(Path::new("C:\\").join("test"), "C:\\test");
}

#[test]
fn with_file_name() {
    same_path(Path::new("C:\\Users\\Something.txt\\").with_file_name("test"), "C:\\Users\\test");
    same_path(Path::new("C:\\Users\\Something.txt/").with_file_name("test"), "C:\\Users\\test");
    same_path(Path::new("C:\\Users\\Something.txt").with_file_name("test"), "C:\\Users\\test");
    same_path(Path::new("C:\\").with_file_name("test"), "C:\\test");
}

#[test]
fn with_extension() {
    same_path(Path::new("C:\\Users\\Something.txt\\").with_extension("test"), "C:\\Users\\Something.test");
    same_path(Path::new("C:\\Users\\Something.txt/").with_extension("test"), "C:\\Users\\Something.test");
    same_path(Path::new("C:\\Users\\Something.txt").with_extension("test"), "C:\\Users\\Something.test");
    same_path(Path::new("C:\\Users\\Something").with_extension("test"), "C:\\Users\\Something.test");
    same_path(Path::new("C:\\").with_extension("test"), "C:\\");
}

#[test]
fn components() {
    let mut c = Path::new("C:\\Users\\Something\\/////./././././/.//\\\\\\\\..").components();
    assert_eq!(c.next().unwrap(), "C:");
    assert_eq!(c.next().unwrap(), "Users");
    assert_eq!(c.next().unwrap(), "Something");
    assert_eq!(c.next().unwrap(), "..");
    assert_eq!(c.next(), None);

    let mut c = Path::new("C:\\Users\\Something\\/////./././././/.//\\\\\\\\..").components();
    assert_eq!(c.next_back().unwrap(), "..");
    assert_eq!(c.next_back().unwrap(), "Something");
    assert_eq!(c.next_back().unwrap(), "Users");
    assert_eq!(c.next_back().unwrap(), "C:");
    assert_eq!(c.next_back(), None);
}

#[test]
fn iter() {
    let mut c = Path::new("C:\\Users\\Something\\/////./././././/.//\\\\\\\\..").iter();
    assert_eq!(c.next().unwrap(), "C:");
    assert_eq!(c.next().unwrap(), "Users");
    assert_eq!(c.next().unwrap(), "Something");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), ".");
    assert_eq!(c.next().unwrap(), ".");
    assert_eq!(c.next().unwrap(), ".");
    assert_eq!(c.next().unwrap(), ".");
    assert_eq!(c.next().unwrap(), ".");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), ".");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), "");
    assert_eq!(c.next().unwrap(), "..");
    assert_eq!(c.next(), None);

    let mut c = Path::new("C:\\Users\\Something\\/////./././././/.//\\\\\\\\..").iter();
    assert_eq!(c.next_back().unwrap(), "..");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), ".");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), ".");
    assert_eq!(c.next_back().unwrap(), ".");
    assert_eq!(c.next_back().unwrap(), ".");
    assert_eq!(c.next_back().unwrap(), ".");
    assert_eq!(c.next_back().unwrap(), ".");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), "");
    assert_eq!(c.next_back().unwrap(), "Something");
    assert_eq!(c.next_back().unwrap(), "Users");
    assert_eq!(c.next_back().unwrap(), "C:");
    assert_eq!(c.next_back(), None);
}
