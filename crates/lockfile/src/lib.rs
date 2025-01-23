// Inspired from https://github.com/sigp/lighthouse/tree/stable/common/lockfile

use fs2::FileExt;
use std::fs::{self, File};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

/// Cross-platform file lock that auto-deletes on drop.
///
/// This lockfile uses OS locking primitives (`flock` on Unix, `LockFile` on Windows), and will
/// only fail if locked by another process. If the file already exists but isn't locked, it can
/// still be locked. This is relevant if an ungraceful shutdown caused the lockfile not to be deleted.
#[derive(Debug)]
pub struct Lockfile {
    #[allow(dead_code)]
    file: File,
    path: PathBuf,
    file_existed: bool,
}

/// Errors that can occur when working with the Lockfile.
#[derive(Debug)]
pub enum LockfileError {
    FileLocked(PathBuf),
    IoError(PathBuf, io::Error),
    UnableToOpenFile(PathBuf, io::Error),
}

impl Lockfile {
    /// Creates and locks a new lockfile at `path`, creating it if it doesn't exist.
    pub fn new(path: PathBuf) -> Result<Self, LockfileError> {
        let file_existed = path.exists();
        let file = match Self::open_or_create_file(&path, file_existed) {
            Ok(f) => f,
            Err(e) => return Err(LockfileError::UnableToOpenFile(path.clone(), e)),
        };

        file.try_lock_exclusive().map_err(|e| match e.kind() {
            ErrorKind::WouldBlock => LockfileError::FileLocked(path.clone()),
            _ => LockfileError::IoError(path.clone(), e),
        })?;

        Ok(Self {
            file,
            path,
            file_existed,
        })
    }

    /// Opens the file at `path` if it exists, or creates a new file if it doesn't.
    fn open_or_create_file(path: &Path, file_existed: bool) -> io::Result<File> {
        if file_existed {
            File::open(path)
        } else {
            File::options()
                .read(true)
                .write(true)
                .create_new(true)
                .open(path)
        }
    }

    /// Returns `true` if the lockfile already existed when the lock was created.
    pub fn file_existed(&self) -> bool {
        self.file_existed
    }

    /// Returns the path of the lockfile.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Lockfile {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            eprintln!("Failed to delete lockfile at {:?}: {}", self.path, e);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(unix)]
    use std::{fs::Permissions, os::unix::fs::PermissionsExt};
    use tempfile::tempdir;

    #[test]
    fn test_lockfile_creation_and_locking() {
        let temp_dir = tempdir().unwrap();
        let lockfile_path = temp_dir.path().join("lockfile");

        let _lock = Lockfile::new(lockfile_path.clone()).unwrap();

        if cfg!(windows) {
            // On Windows, attempting to reopen the lockfile results in an IoError since it's already open.
            assert!(matches!(
                Lockfile::new(lockfile_path).unwrap_err(),
                LockfileError::IoError(..),
            ));
        } else {
            // On Unix, the lockfile should report as locked.
            assert!(matches!(
                Lockfile::new(lockfile_path).unwrap_err(),
                LockfileError::FileLocked(..),
            ));
        }
    }

    #[test]
    fn test_relocking_after_drop() {
        let temp_dir = tempdir().unwrap();
        let lockfile_path = temp_dir.path().join("lockfile");

        let lock1 = Lockfile::new(lockfile_path.clone()).unwrap();
        drop(lock1); // Unlocks and deletes the lockfile.

        let lock2 = Lockfile::new(lockfile_path.clone()).unwrap();
        assert!(!lock2.file_existed());
        drop(lock2);

        // Ensure the lockfile was deleted after being dropped.
        assert!(!lockfile_path.exists());
    }

    #[test]
    fn test_existing_lockfile_detection() {
        let temp_dir = tempdir().unwrap();
        let lockfile_path = temp_dir.path().join("lockfile");

        File::create(&lockfile_path).unwrap();

        let lock = Lockfile::new(lockfile_path).unwrap();
        assert!(lock.file_existed());
    }

    #[test]
    #[cfg(unix)]
    fn test_permission_denied_on_create() {
        let temp_dir = tempdir().unwrap();
        let lockfile_path = temp_dir.path().join("lockfile");

        let lockfile = File::create(&lockfile_path).unwrap();
        lockfile
            .set_permissions(Permissions::from_mode(0o000))
            .unwrap();

        assert!(matches!(
            Lockfile::new(lockfile_path).unwrap_err(),
            LockfileError::UnableToOpenFile(..)
        ));
    }
}
