use std::{io, path::Path};

use io_uring::{opcode, types};

use crate::complete::complete;

use super::util::to_cstring;

pub async fn unlink(path: impl AsRef<Path>, flags: i32) -> io::Result<()> {
    let path = to_cstring(path)?;
    let entry = opcode::UnlinkAt::new(types::Fd(libc::AT_FDCWD), path.as_ptr())
        .flags(flags)
        .build();
    complete(entry).await?;
    Ok(())
}

pub async fn unlink_file(path: impl AsRef<Path>) -> io::Result<()> {
    unlink(path, 0).await
}

pub async fn unlink_dir(path: impl AsRef<Path>) -> io::Result<()> {
    unlink(path, libc::AT_REMOVEDIR).await
}

pub async fn symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    let from = to_cstring(from)?;
    let to = to_cstring(to)?;
    let entry =
        opcode::SymlinkAt::new(types::Fd(libc::AT_FDCWD), from.as_ptr(), to.as_ptr()).build();
    complete(entry).await?;
    Ok(())
}
