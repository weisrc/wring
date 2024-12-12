use std::{io, path::Path};

use io_uring::{opcode, types};

use crate::{complete, io::Fd};

use super::util::to_cstring;

pub async fn open(path: impl AsRef<Path>) -> io::Result<Fd> {
    let path = to_cstring(path)?;
    let entry = opcode::OpenAt::new(types::Fd(libc::AT_FDCWD), path.as_ptr()).build();
    let out = complete(entry).await?;
    Ok(Fd::new(out.result()))
}

pub async fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    let from = to_cstring(from)?;
    let to = to_cstring(to)?;
    let entry = opcode::RenameAt::new(
        types::Fd(libc::AT_FDCWD),
        from.as_ptr(),
        types::Fd(libc::AT_FDCWD),
        to.as_ptr(),
    )
    .build();
    complete(entry).await?;
    Ok(())
}

pub async fn mkdir(path: impl AsRef<Path>, mode: u32) -> io::Result<()> {
    let path = to_cstring(path)?;
    let entry = opcode::MkDirAt::new(types::Fd(libc::AT_FDCWD), path.as_ptr())
        .mode(mode)
        .build();
    complete(entry).await?;
    Ok(())
}
