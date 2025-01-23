use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

#[cfg(windows)]
use winapi::um::winnt::{FILE_GENERIC_READ, FILE_GENERIC_WRITE, STANDARD_RIGHTS_ALL};

/// This is the security identifier in Windows for the owner of a file. See:
/// - https://docs.microsoft.com/en-us/troubleshoot/windows-server/identity/security-identifiers-in-windows#well-known-sids-all-versions-of-windows
#[cfg(windows)]
const OWNER_SID_STR: &str = "S-1-3-4";
/// We don't need any of the `AceFlags` listed here:
/// - https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-ace_header
#[cfg(windows)]
const OWNER_ACL_ENTRY_FLAGS: u8 = 0;
/// Generic Rights:
///  - https://docs.microsoft.com/en-us/windows/win32/fileio/file-security-and-access-rights
/// Individual Read/Write/Execute Permissions (referenced in generic rights link):
///  - https://docs.microsoft.com/en-us/windows/win32/wmisdk/file-and-directory-access-rights-constants
/// STANDARD_RIGHTS_ALL
///  - https://docs.microsoft.com/en-us/windows/win32/secauthz/access-mask
#[cfg(windows)]
const OWNER_ACL_ENTRY_MASK: u32 = FILE_GENERIC_READ | FILE_GENERIC_WRITE | STANDARD_RIGHTS_ALL;

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("The file could not be created: {0}")]
    /// The file could not be created
    UnableToCreateFile(io::Error),
    #[error("The file could not be copied: {0}")]
    /// The file could not be copied
    UnableToCopyFile(io::Error),
    #[error("The file could not be opened: {0}")]
    /// The file could not be opened
    UnableToOpenFile(io::Error),
    #[error("The file could not be renamed: {0}")]
    /// The file could not be renamed
    UnableToRenameFile(io::Error),
    #[error("Failed to set permissions: {0}")]
    /// Failed to set permissions
    UnableToSetPermissions(io::Error),
    #[error("Failed to retrieve file metadata: {0}")]
    /// Failed to retrieve file metadata
    UnableToRetrieveMetadata(io::Error),
    #[error("Failed to write bytes to file: {0}")]
    /// Failed to write bytes to file
    UnableToWriteFile(io::Error),
    #[error("Failed to obtain file path")]
    /// Failed to obtain file path
    UnableToObtainFilePath,
    #[error("Failed to convert string to SID: {0}")]
    /// Failed to convert string to SID
    UnableToConvertSID(u32),
    #[error("Failed to retrieve ACL for file: {0}")]
    /// Failed to retrieve ACL for file
    UnableToRetrieveACL(u32),
    #[error("Failed to enumerate ACL entries: {0}")]
    /// Failed to enumerate ACL entries
    UnableToEnumerateACLEntries(u32),
    #[error("Failed to add new ACL entry: {0}")]
    /// Failed to add new ACL entry
    UnableToAddACLEntry(String),
    #[error("Failed to remove ACL entry: {0}")]
    /// Failed to remove ACL entry
    UnableToRemoveACLEntry(String),
}

/// Write a file atomically by using a temporary file as an intermediate.
///
/// Care is taken to preserve the permissions of the file at `file_path` being written.
///
/// If no file exists at `file_path` one will be created with restricted 0o600-equivalent
/// permissions.
pub fn write_file_via_temporary(
    file_path: &Path,
    temp_path: &Path,
    bytes: &[u8],
) -> Result<(), FsError> {
    // If the file already exists, preserve its permissions by copying it.
    // Otherwise, create a new file with restricted permissions.
    if file_path.exists() {
        fs::copy(file_path, temp_path).map_err(FsError::UnableToCopyFile)?;
        fs::write(temp_path, bytes).map_err(FsError::UnableToWriteFile)?;
    } else {
        create_with_600_perms(temp_path, bytes)?;
    }

    // With the temporary file created, perform an atomic rename.
    fs::rename(temp_path, file_path).map_err(FsError::UnableToRenameFile)?;

    Ok(())
}

/// Creates a file with `600 (-rw-------)` permissions and writes the specified bytes to file.
pub fn create_with_600_perms<P: AsRef<Path>>(path: P, bytes: &[u8]) -> Result<(), FsError> {
    let path = path.as_ref();
    let mut file = File::create(path).map_err(FsError::UnableToCreateFile)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = file
            .metadata()
            .map_err(FsError::UnableToRetrieveMetadata)?
            .permissions();
        perm.set_mode(0o600);
        file.set_permissions(perm)
            .map_err(FsError::UnableToSetPermissions)?;
    }

    file.write_all(bytes).map_err(FsError::UnableToWriteFile)?;
    #[cfg(windows)]
    {
        restrict_file_permissions(path)?;
    }

    Ok(())
}

pub fn restrict_file_permissions<P: AsRef<Path>>(path: P) -> Result<(), FsError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file = File::open(path.as_ref()).map_err(FsError::UnableToOpenFile)?;
        let mut perm = file
            .metadata()
            .map_err(FsError::UnableToRetrieveMetadata)?
            .permissions();
        perm.set_mode(0o600);
        file.set_permissions(perm)
            .map_err(FsError::UnableToSetPermissions)?;
    }

    #[cfg(windows)]
    {
        use winapi::um::winnt::PSID;
        use windows_acl::acl::{AceType, ACL};
        use windows_acl::helper::sid_to_string;

        let path_str = path
            .as_ref()
            .to_str()
            .ok_or(Error::UnableToObtainFilePath)?;
        let mut acl = ACL::from_file_path(path_str, false).map_err(Error::UnableToRetrieveACL)?;

        let owner_sid =
            windows_acl::helper::string_to_sid(OWNER_SID_STR).map_err(Error::UnableToConvertSID)?;

        let entries = acl.all().map_err(Error::UnableToEnumerateACLEntries)?;

        // add single entry for file owner
        acl.add_entry(
            owner_sid.as_ptr() as PSID,
            AceType::AccessAllow,
            OWNER_ACL_ENTRY_FLAGS,
            OWNER_ACL_ENTRY_MASK,
        )
        .map_err(|code| {
            Error::UnableToAddACLEntry(format!(
                "Failed to add ACL entry for SID {} error={}",
                OWNER_SID_STR, code
            ))
        })?;
        // remove all AccessAllow entries from the file that aren't the owner_sid
        for entry in &entries {
            if let Some(ref entry_sid) = entry.sid {
                let entry_sid_str = sid_to_string(entry_sid.as_ptr() as PSID)
                    .unwrap_or_else(|_| "BadFormat".to_string());
                if entry_sid_str != OWNER_SID_STR {
                    acl.remove(entry_sid.as_ptr() as PSID, Some(AceType::AccessAllow), None)
                        .map_err(|_| {
                            Error::UnableToRemoveACLEntry(format!(
                                "Failed to remove ACL entry for SID {}",
                                entry_sid_str
                            ))
                        })?;
                }
            }
        }
    }

    Ok(())
}
