use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// This is used to implement `ToString` for non-crate types.
/// It is mainly used for types like `Path`, to eliminate the extra steps introduced by the
/// `to_string_lossy` method, as we are not that concerned with correctness in these settings.
pub trait ToStringHelper {
    fn to_string_helper(&self) -> String;
}

impl ToString for dyn ToStringHelper {
    fn to_string(&self) -> String {
        self.to_string_helper()
    }
}

impl ToStringHelper for Path {
    fn to_string_helper(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}

impl ToStringHelper for PathBuf {
    fn to_string_helper(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}

impl ToStringHelper for OsStr {
    fn to_string_helper(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}

impl ToStringHelper for OsString {
    fn to_string_helper(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}
