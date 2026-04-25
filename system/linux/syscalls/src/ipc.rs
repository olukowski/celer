#![cfg(target_arch = "x86")]

use celer_system_linux_ctypes::{Int, Long, UnsignedInt, Void};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

pub use crate::sys::{
    MSGCTL, MSGGET, MSGRCV, MSGSND, SEMCTL, SEMGET, SEMOP, SHMAT, SHMCTL,
    SHMDT, SHMGET,
};

/// Errors returned by [`ipc`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IpcError {
    /// `EINVAL`.
    Einval,
    /// `ENOSYS`.
    Enosys,
    /// Another errno returned by the selected SysV IPC helper.
    Other(Errno),
}

impl IpcError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Einval => Self::Einval,
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

/// Dispatch a Linux 1.0 SysV IPC operation through the historical `ipc(2)`
/// multiplexor.
///
/// This wrapper keeps the original multiplexed ABI shape and maps the raw
/// return value into `Result<Long, IpcError>`.
///
/// On success, returns the selected helper's nonnegative raw result without
/// further interpretation.
///
/// See [`sys::ipc`] for kernel behavior, selector semantics, reachable
/// errors, and source references.
///
/// # Safety
/// - Depending on `call`, `ptr` may be treated as a userspace pointer that the
///   kernel reads from, writes to, or both. The caller must uphold the
///   selected subcall's pointer validity requirements.
/// - When `call == MSGRCV`, `ptr` must point to a readable Linux 1.0
///   `ipc_kludge` record.
/// - When `call == SHMAT`, `third` is interpreted by the kernel as a user
///   output-pointer argument encoded in the raw multiplexed ABI.
///
/// # Errors
/// - [`IpcError::Einval`]: `call` was not a supported selector, or the entry
///   path rejected the selected argument shape.
/// - [`IpcError::Enosys`]: the running kernel left SysV IPC support disabled
///   for this multiplexed entry point.
/// - [`IpcError::Other`]: another errno returned by the selected semaphore,
///   message-queue, or shared-memory helper.
pub unsafe fn ipc(
    call: UnsignedInt,
    first: Int,
    second: Int,
    third: Int,
    ptr: *mut Void,
) -> Result<Long, IpcError> {
    // SAFETY: the caller must uphold the selected subcall's pointer and ABI
    // requirements for `ptr` and, for `SHMAT`, the integer-encoded output
    // pointer carried in `third`.
    let ret = unsafe { sys::ipc(call, first, second, third, ptr) };

    result_from_ret(ret as isize, |ret| ret as Long, IpcError::from_errno)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::Errno;

    use super::{IpcError, MSGRCV, ipc};

    #[test]
    fn test_ipc_invalid_selector() {
        let err = unsafe { ipc(0, 0, 0, 0, core::ptr::null_mut()) }
            .expect_err("selector 0 should be rejected by ipc");

        assert!(matches!(err, IpcError::Einval | IpcError::Enosys));
    }

    #[test]
    fn test_ipc_msgrcv_null_kludge() {
        let err = unsafe { ipc(MSGRCV, 0, 0, 0, core::ptr::null_mut()) }
            .expect_err("ipc(MSGRCV, ..., null) should be rejected");

        assert!(matches!(err, IpcError::Einval | IpcError::Enosys));
    }

    #[test]
    fn test_ipc_error_mapping() {
        assert_eq!(IpcError::from_errno(Errno::Einval), IpcError::Einval);
        assert_eq!(IpcError::from_errno(Errno::Enosys), IpcError::Enosys);
        assert_eq!(
            IpcError::from_errno(Errno::Efault),
            IpcError::Other(Errno::Efault)
        );
    }
}
