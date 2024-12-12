use std::{ffi::CString, path::Path};

pub(crate) fn to_cstring(path: impl AsRef<Path>) -> std::io::Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    let path = CString::new(path.as_ref().as_os_str().as_bytes())?;
    Ok(path)
}