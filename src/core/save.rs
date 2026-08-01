use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Result;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(windows)]
#[allow(non_snake_case)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn MoveFileExW(existing_file_name: *const u16, new_file_name: *const u16, flags: u32) -> i32;
}

/// Strategy used when saving a settings file.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SaveMode {
    /// Write the new JSON to a temporary file, flush it, and replace the target.
    ///
    /// This is the default mode. It avoids exposing a partially written final
    /// settings file if the process stops during the write step.
    Atomic,

    /// Write JSON directly to the final path.
    ///
    /// This mode matches the v2.2.0 behavior and is useful for unusual
    /// filesystems, debugging, or applications that intentionally want direct
    /// overwrite semantics.
    Direct,
}

pub fn save_to_path(path: &Path, content: &str, mode: SaveMode) -> Result<()> {
    if let Some(parent) = non_empty_parent(path) {
        fs::create_dir_all(parent)?;
    }

    match mode {
        SaveMode::Atomic => save_atomic(path, content),
        SaveMode::Direct => save_direct(path, content),
    }
}

fn save_direct(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)?;
    Ok(())
}

fn save_atomic(path: &Path, content: &str) -> Result<()> {
    let (temp_path, mut temp_file) = create_temp_file(path)?;

    let result = (|| -> io::Result<()> {
        temp_file.write_all(content.as_bytes())?;
        temp_file.sync_all()?;
        drop(temp_file);

        replace_file(&temp_path, path)?;
        sync_parent_dir(path);
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    result?;
    Ok(())
}

fn create_temp_file(target: &Path) -> io::Result<(PathBuf, File)> {
    let parent = non_empty_parent(target).unwrap_or_else(|| Path::new("."));
    let target_name = target
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "settings.json".into());

    for attempt in 0..1000_u16 {
        let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let temp_name = format!(
            ".{target_name}.tmp.{}.{}.{}.{}",
            std::process::id(),
            nanos,
            counter,
            attempt
        );
        let temp_path = parent.join(temp_name);

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique temporary settings file",
    ))
}

#[cfg(unix)]
fn replace_file(temp_path: &Path, target_path: &Path) -> io::Result<()> {
    fs::rename(temp_path, target_path)
}

#[cfg(windows)]
fn replace_file(temp_path: &Path, target_path: &Path) -> io::Result<()> {
    const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

    let old_path = wide_null_terminated(temp_path);
    let new_path = wide_null_terminated(target_path);

    // SAFETY: Both pointers are valid, null-terminated UTF-16 buffers that live
    // for the duration of the call. The flags request an in-place replacement
    // of the destination by a temporary file created in the same directory.
    let ok = unsafe {
        MoveFileExW(
            old_path.as_ptr(),
            new_path.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };

    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(windows)]
fn wide_null_terminated(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0]).collect()
}

#[cfg(not(any(unix, windows)))]
fn replace_file(temp_path: &Path, target_path: &Path) -> io::Result<()> {
    if target_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "atomic replacement is not implemented for this target; use SaveMode::Direct",
        ));
    }

    fs::rename(temp_path, target_path)
}

fn non_empty_parent(path: &Path) -> Option<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

fn sync_parent_dir(path: &Path) {
    if let Some(parent) = non_empty_parent(path) {
        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }
    }
}
