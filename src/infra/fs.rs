use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn resolve_out_path(p: &str) -> Result<PathBuf> {
    let pb = Path::new(p).to_path_buf();
    if pb.is_dir() {
        anyhow::bail!("out path is a directory");
    }
    Ok(pb)
}

pub fn write_atomic<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<()> {
    // own a PathBuf so we can create the tmp path and move the final PathBuf into rename
    let pbuf = path.as_ref().to_path_buf();
    let tmp = pbuf.with_extension("tmp");
    let mut f = fs::File::create(&tmp)?;
    f.write_all(content)?;
    f.sync_all()?;
    // move pbuf (no needless borrow)
    fs::rename(&tmp, pbuf)?;
    Ok(())
}
