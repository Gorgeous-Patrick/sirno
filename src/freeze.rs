//! Cross-platform immutable file operations.
//!
//! Sirno uses this module as the local enforcement layer for frozen public files.
//! The public interface is platform-neutral.
//! Each backend uses the strongest ordinary filesystem primitive available on that platform.

use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use thiserror::Error;
use tracing::trace;

/// A path whose mutability can be controlled by Sirno.
///
/// Symbolic links are ignored.
/// Sirno freezes the link target only through an explicit path to that target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrozenPath {
    path: PathBuf,
}

impl FrozenPath {
    /// Construct an immutable-operation handle for `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Return the path controlled by this handle.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Freeze this path.
    ///
    /// The visible read-only permission is applied before the stronger platform guard.
    pub fn freeze(&self) -> Result<(), FreezeError> {
        trace!("freeze path begin: path={}", self.path.display());
        let Some(metadata) = self.target_metadata()? else {
            trace!("freeze path end: symlink skipped");
            return Ok(());
        };
        let original_permissions = metadata.permissions();
        set_permission_hint(&self.path, &metadata, false)?;
        if let Err(source) = platform::set_immutable(&self.path, true) {
            let _ = fs::set_permissions(&self.path, original_permissions);
            return Err(FreezeError::SetImmutable { path: self.path.clone(), source });
        }
        trace!("freeze path end: path={}", self.path.display());
        Ok(())
    }

    /// Melt this path so normal writes can resume.
    ///
    /// The stronger platform guard is cleared before visible write permission is restored.
    pub fn melt(&self) -> Result<(), FreezeError> {
        trace!("melt path begin: path={}", self.path.display());
        let Some(_) = self.target_metadata()? else {
            trace!("melt path end: symlink skipped");
            return Ok(());
        };
        platform::set_immutable(&self.path, false)
            .map_err(|source| FreezeError::SetImmutable { path: self.path.clone(), source })?;
        let metadata = self.target_metadata()?.expect("path was checked before melting");
        set_permission_hint(&self.path, &metadata, true)?;
        trace!("melt path end: path={}", self.path.display());
        Ok(())
    }

    /// Return whether this path currently carries Sirno's platform immutable state.
    pub fn is_frozen(&self) -> Result<bool, FreezeError> {
        let Some(_) = self.target_metadata()? else {
            return Ok(false);
        };
        platform::is_immutable(&self.path)
            .map_err(|source| FreezeError::ReadImmutable { path: self.path.clone(), source })
    }

    fn target_metadata(&self) -> Result<Option<Metadata>, FreezeError> {
        let metadata = fs::symlink_metadata(&self.path)
            .map_err(|source| FreezeError::Inspect { path: self.path.clone(), source })?;
        if metadata.file_type().is_symlink() {
            return Ok(None);
        }
        Ok(Some(metadata))
    }
}

fn set_permission_hint(
    path: &Path, metadata: &Metadata, writable: bool,
) -> Result<(), FreezeError> {
    let mut permissions = metadata.permissions();
    set_permissions_writable(&mut permissions, metadata.file_type().is_dir(), writable);
    fs::set_permissions(path, permissions)
        .map_err(|source| FreezeError::SetPermissions { path: path.to_path_buf(), source })
}

#[cfg(unix)]
fn set_permissions_writable(permissions: &mut fs::Permissions, is_directory: bool, writable: bool) {
    let mode = permissions.mode();
    let next = if writable {
        if is_directory { mode | 0o700 } else { mode | 0o600 }
    } else {
        mode & !0o222
    };
    permissions.set_mode(next);
}

#[cfg(not(unix))]
fn set_permissions_writable(
    permissions: &mut fs::Permissions, _is_directory: bool, writable: bool,
) {
    permissions.set_readonly(!writable);
}

/// Error raised while changing a path's freeze state.
#[derive(Debug, Error)]
pub enum FreezeError {
    /// The path could not be inspected before an operation.
    #[error("failed to inspect freeze path {path}")]
    Inspect {
        /// Path being inspected.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },
    /// The path could not be represented for a platform call.
    #[error("freeze path contains an interior NUL byte: {0}")]
    InteriorNul(PathBuf),
    /// The visible read-only or writable permission hint could not be changed.
    #[error("failed to update visible file permissions for {path}")]
    SetPermissions {
        /// Path whose permission hint could not be changed.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },
    /// The platform immutable state could not be read.
    #[error("failed to read immutable state for {path}")]
    ReadImmutable {
        /// Path whose immutable state could not be read.
        path: PathBuf,
        /// Underlying platform error.
        #[source]
        source: io::Error,
    },
    /// The platform immutable state could not be changed.
    #[error("failed to update immutable state for {path}")]
    SetImmutable {
        /// Path whose immutable state could not be changed.
        path: PathBuf,
        /// Underlying platform error.
        #[source]
        source: io::Error,
    },
}

#[cfg(target_os = "linux")]
mod platform {
    use std::fs::File;
    use std::io;
    use std::path::Path;

    use rustix::fs::{IFlags, ioctl_getflags, ioctl_setflags};

    pub fn set_immutable(path: &Path, immutable: bool) -> io::Result<()> {
        let file = File::open(path)?;
        let mut flags = ioctl_getflags(&file).map_err(io::Error::from)?;
        flags.set(IFlags::IMMUTABLE, immutable);
        ioctl_setflags(&file, flags).map_err(io::Error::from)
    }

    pub fn is_immutable(path: &Path) -> io::Result<bool> {
        let file = File::open(path)?;
        let flags = ioctl_getflags(&file).map_err(io::Error::from)?;
        Ok(flags.contains(IFlags::IMMUTABLE))
    }
}

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod platform {
    use std::ffi::CString;
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    use crate::freeze::FreezeError;

    #[cfg(any(target_os = "macos", target_os = "openbsd"))]
    type FileFlags = libc::c_uint;
    #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
    type FileFlags = libc::c_ulong;

    pub fn set_immutable(path: &Path, immutable: bool) -> io::Result<()> {
        let c_path = c_path(path)?;
        let current = stat_flags(&c_path)?;
        let flag = libc::UF_IMMUTABLE as FileFlags;
        let next = if immutable { current | flag } else { current & !flag };
        if unsafe { libc::chflags(c_path.as_ptr(), next) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn is_immutable(path: &Path) -> io::Result<bool> {
        let c_path = c_path(path)?;
        Ok(stat_flags(&c_path)? & (libc::UF_IMMUTABLE as FileFlags) != 0)
    }

    fn c_path(path: &Path) -> io::Result<CString> {
        CString::new(path.as_os_str().as_bytes()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                FreezeError::InteriorNul(path.to_path_buf()),
            )
        })
    }

    fn stat_flags(path: &CString) -> io::Result<FileFlags> {
        let mut stat = MaybeUninit::<libc::stat>::uninit();
        if unsafe { libc::stat(path.as_ptr(), stat.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { stat.assume_init() }.st_flags)
    }
}

#[cfg(windows)]
mod platform {
    use std::io;
    use std::mem::{MaybeUninit, size_of};
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::ptr::{addr_of, null, null_mut};

    use windows_sys::Win32::Foundation::{ERROR_SUCCESS, GetLastError, LocalFree};
    use windows_sys::Win32::Security::Authorization::{
        BuildTrusteeWithSidW, DENY_ACCESS, EXPLICIT_ACCESS_W, GetNamedSecurityInfoW,
        SE_FILE_OBJECT, SetEntriesInAclW, SetNamedSecurityInfoW, TRUSTEE_W,
    };
    use windows_sys::Win32::Security::{
        ACCESS_DENIED_ACE, ACE_HEADER, ACL, ACL_SIZE_INFORMATION, AclSizeInformation,
        CreateWellKnownSid, DACL_SECURITY_INFORMATION, DeleteAce, EqualSid, GetAce,
        GetAclInformation, NO_INHERITANCE, PSECURITY_DESCRIPTOR, PSID, SECURITY_MAX_SID_SIZE,
        WinWorldSid,
    };
    use windows_sys::Win32::Storage::FileSystem::{DELETE, FILE_DELETE_CHILD, FILE_GENERIC_WRITE};
    use windows_sys::core::PCWSTR;

    const ACCESS_DENIED_ACE_TYPE: u8 = 1;
    const FREEZE_MASK: u32 = FILE_GENERIC_WRITE | DELETE | FILE_DELETE_CHILD;

    pub fn set_immutable(path: &Path, immutable: bool) -> io::Result<()> {
        if immutable {
            remove_freeze_ace(path)?;
            add_freeze_ace(path)
        } else {
            remove_freeze_ace(path)
        }
    }

    pub fn is_immutable(path: &Path) -> io::Result<bool> {
        let sid = world_sid()?;
        let wide = wide_path(path);
        let descriptor = SecurityDescriptor::read(wide.as_ptr())?;
        Ok(unsafe { dacl_contains_freeze_ace(descriptor.dacl, sid.as_ptr() as PSID)? })
    }

    fn add_freeze_ace(path: &Path) -> io::Result<()> {
        let sid = world_sid()?;
        let wide = wide_path(path);
        let descriptor = SecurityDescriptor::read(wide.as_ptr())?;
        let mut trustee = TRUSTEE_W::default();
        unsafe { BuildTrusteeWithSidW(&mut trustee, sid.as_ptr() as PSID) };
        let explicit_access = EXPLICIT_ACCESS_W {
            grfAccessPermissions: FREEZE_MASK,
            grfAccessMode: DENY_ACCESS,
            grfInheritance: NO_INHERITANCE,
            Trustee: trustee,
        };
        let mut new_acl = null_mut::<ACL>();
        let status =
            unsafe { SetEntriesInAclW(1, &explicit_access, descriptor.dacl, &mut new_acl) };
        if status != ERROR_SUCCESS {
            return Err(win32_error(status));
        }
        let set_result = unsafe {
            SetNamedSecurityInfoW(
                wide.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                new_acl,
                null(),
            )
        };
        unsafe { LocalFree(new_acl as _) };
        if set_result != ERROR_SUCCESS {
            return Err(win32_error(set_result));
        }
        Ok(())
    }

    fn remove_freeze_ace(path: &Path) -> io::Result<()> {
        let sid = world_sid()?;
        let wide = wide_path(path);
        let descriptor = SecurityDescriptor::read(wide.as_ptr())?;
        let changed =
            unsafe { remove_matching_freeze_aces(descriptor.dacl, sid.as_ptr() as PSID)? };
        if !changed {
            return Ok(());
        }
        let status = unsafe {
            SetNamedSecurityInfoW(
                wide.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                descriptor.dacl,
                null(),
            )
        };
        if status != ERROR_SUCCESS {
            return Err(win32_error(status));
        }
        Ok(())
    }

    unsafe fn remove_matching_freeze_aces(dacl: *mut ACL, sid: PSID) -> io::Result<bool> {
        if dacl.is_null() {
            return Ok(false);
        }
        let mut changed = false;
        for index in (0..acl_ace_count(dacl)?).rev() {
            let mut ace = null_mut();
            if GetAce(dacl, index, &mut ace) == 0 {
                return Err(last_error());
            }
            if is_freeze_ace(ace, sid) {
                if DeleteAce(dacl, index) == 0 {
                    return Err(last_error());
                }
                changed = true;
            }
        }
        Ok(changed)
    }

    unsafe fn dacl_contains_freeze_ace(dacl: *mut ACL, sid: PSID) -> io::Result<bool> {
        if dacl.is_null() {
            return Ok(false);
        }
        for index in 0..acl_ace_count(dacl)? {
            let mut ace = null_mut();
            if GetAce(dacl, index, &mut ace) == 0 {
                return Err(last_error());
            }
            if is_freeze_ace(ace, sid) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    unsafe fn acl_ace_count(dacl: *mut ACL) -> io::Result<u32> {
        let mut info = MaybeUninit::<ACL_SIZE_INFORMATION>::zeroed();
        if GetAclInformation(
            dacl,
            info.as_mut_ptr().cast(),
            size_of::<ACL_SIZE_INFORMATION>() as u32,
            AclSizeInformation,
        ) == 0
        {
            return Err(last_error());
        }
        Ok(info.assume_init().AceCount)
    }

    unsafe fn is_freeze_ace(ace: *mut core::ffi::c_void, sid: PSID) -> bool {
        let header = &*(ace as *const ACE_HEADER);
        if header.AceType != ACCESS_DENIED_ACE_TYPE || header.AceFlags != NO_INHERITANCE as u8 {
            return false;
        }
        let denied = &*(ace as *const ACCESS_DENIED_ACE);
        if denied.Mask != FREEZE_MASK {
            return false;
        }
        let ace_sid = addr_of!(denied.SidStart) as PSID;
        EqualSid(ace_sid, sid) != 0
    }

    fn world_sid() -> io::Result<Vec<u8>> {
        let mut sid = vec![0_u8; SECURITY_MAX_SID_SIZE as usize];
        let mut size = sid.len() as u32;
        if unsafe {
            CreateWellKnownSid(WinWorldSid, null_mut(), sid.as_mut_ptr() as PSID, &mut size)
        } == 0
        {
            return Err(last_error());
        }
        sid.truncate(size as usize);
        Ok(sid)
    }

    fn wide_path(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    }

    fn win32_error(code: u32) -> io::Error {
        io::Error::from_raw_os_error(code as i32)
    }

    fn last_error() -> io::Error {
        unsafe { win32_error(GetLastError()) }
    }

    struct SecurityDescriptor {
        descriptor: PSECURITY_DESCRIPTOR,
        dacl: *mut ACL,
    }

    impl SecurityDescriptor {
        fn read(path: PCWSTR) -> io::Result<Self> {
            let mut dacl = null_mut::<ACL>();
            let mut descriptor = null_mut();
            let status = unsafe {
                GetNamedSecurityInfoW(
                    path,
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    null_mut(),
                    null_mut(),
                    &mut dacl,
                    null_mut(),
                    &mut descriptor,
                )
            };
            if status != ERROR_SUCCESS {
                return Err(win32_error(status));
            }
            Ok(Self { descriptor, dacl })
        }
    }

    impl Drop for SecurityDescriptor {
        fn drop(&mut self) {
            unsafe {
                LocalFree(self.descriptor as _);
            }
        }
    }
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    windows
)))]
mod platform {
    use std::io;
    use std::path::Path;

    pub fn set_immutable(_path: &Path, _immutable: bool) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Sirno immutable paths are not supported on this platform",
        ))
    }

    pub fn is_immutable(_path: &Path) -> io::Result<bool> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Sirno immutable paths are not supported on this platform",
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn symlink_freeze_is_noop() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let temp = tempfile::tempdir().unwrap();
            let target = temp.path().join("target.md");
            let link = temp.path().join("link.md");
            fs::write(&target, "body").unwrap();
            symlink(&target, &link).unwrap();

            FrozenPath::new(&link).freeze().unwrap();
            assert!(!FrozenPath::new(&link).is_frozen().unwrap());
        }
    }
}
