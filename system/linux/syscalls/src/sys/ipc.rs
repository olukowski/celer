use celer_system_linux_ctypes::{Int, Long, UnsignedInt, Void};

use crate::arch::current::{Sysno, syscall5};

/// Linux 1.0 `ipc(2)` subcall selector for `semop(2)`.
pub const SEMOP: UnsignedInt = 1;
/// Linux 1.0 `ipc(2)` subcall selector for `semget(2)`.
pub const SEMGET: UnsignedInt = 2;
/// Linux 1.0 `ipc(2)` subcall selector for `semctl(2)`.
pub const SEMCTL: UnsignedInt = 3;
/// Linux 1.0 `ipc(2)` subcall selector for `msgsnd(2)`.
pub const MSGSND: UnsignedInt = 11;
/// Linux 1.0 `ipc(2)` subcall selector for `msgrcv(2)`.
pub const MSGRCV: UnsignedInt = 12;
/// Linux 1.0 `ipc(2)` subcall selector for `msgget(2)`.
pub const MSGGET: UnsignedInt = 13;
/// Linux 1.0 `ipc(2)` subcall selector for `msgctl(2)`.
pub const MSGCTL: UnsignedInt = 14;
/// Linux 1.0 `ipc(2)` subcall selector for `shmat(2)`.
pub const SHMAT: UnsignedInt = 21;
/// Linux 1.0 `ipc(2)` subcall selector for `shmdt(2)`.
pub const SHMDT: UnsignedInt = 22;
/// Linux 1.0 `ipc(2)` subcall selector for `shmget(2)`.
pub const SHMGET: UnsignedInt = 23;
/// Linux 1.0 `ipc(2)` subcall selector for `shmctl(2)`.
pub const SHMCTL: UnsignedInt = 24;

/// Dispatch a Linux 1.0 SYSVIPC operation through the historical `ipc(2)`
/// multiplexor.
///
/// This wrapper targets the original Linux 1.0 x86 syscall slot 117 ABI.
///
/// # Safety
/// - Depending on `call`, `ptr` may be treated as a userspace pointer that the
///   kernel reads from, writes to, or both. The caller must uphold the
///   selected subcall's pointer validity requirements for the duration of the
///   syscall.
/// - When `call == MSGRCV`, `ptr` must point to a readable userspace
///   `ipc_kludge` structure whose embedded pointers and message type are valid
///   for the selected receive operation.
/// - When `call == SHMAT`, `third` is interpreted as a userspace
///   `unsigned long *` result pointer. If nonzero, it must be valid for the
///   kernel to write one `unsigned long` result value without violating Rust's
///   aliasing or initialization rules.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 exposes sparse selector values for SysV IPC
///   semaphore, message-queue, and shared-memory helpers; current 32-bit x86
///   kernels still route syscall slot 117 through a compatibility `ipc`
///   multiplexor with additional versioned handling
/// - Availability: present on supported x86 Linux kernels with SysV IPC
///   enabled; kernels built without SysV IPC support reject the entry
///
/// # Required Privileges
/// - Subcall-dependent
///
/// # Behavior
/// - `call` selects one of the Linux 1.0 SysV IPC operations:
///   `semop`, `semget`, `semctl`, `msgsnd`, `msgrcv`, `msgget`, `msgctl`,
///   `shmat`, `shmdt`, `shmget`, or `shmctl`.
/// - This wrapper models the original Linux 1.0 / version-`0` five-argument
///   ABI, not the later six-argument compatibility extensions used by current
///   kernels for some selectors.
/// - `first`, `second`, `third`, and `ptr` are forwarded unchanged to the
///   selected helper according to the historical multiplexed ABI.
/// - `MSGRCV` is special: `ptr` must point to a userspace `ipc_kludge`
///   structure containing the message buffer pointer and message type.
/// - `SHMAT` is special: `third` is interpreted as the userspace `ulong *`
///   output-pointer argument instead of an integer command value.
/// - On success, returns the selected helper's raw result without any
///   multiplexor-level translation.
///
/// # Errors
/// Linux 1.0 entry-path errors:
/// - `EINVAL`: `call` does not match a Linux 1.0 `ipc` selector, or
///   `call == MSGRCV` and `ptr` is null.
/// - `ENOSYS`: the running kernel was built without SysV IPC support.
///
/// All other reachable errors come from the selected semaphore, message-queue,
/// or shared-memory helper rather than from the Linux 1.0 `ipc` entry path
/// itself.
///
/// Current 32-bit x86 kernels keep syscall slot `117` as a compatibility
/// entry, but they add versioned dispatch and additional entry-path failures
/// such as `EFAULT` for some selectors before control reaches the helper.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/ipc.2.html)
/// - Stable:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/ipc/syscall.c?h=v7.0#n11)
/// - LTS:
///   [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/ipc/syscall.c?h=v7.0#n11)
/// - First stable:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/ipc/util.c?h=1.0#n65)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n126)
///
/// # Historical References
/// - Linux 1.0 selectors:
///   [include/linux/ipc.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/ipc.h?h=1.0#n49)
pub unsafe fn ipc(
    call: UnsignedInt,
    first: Int,
    second: Int,
    third: Int,
    ptr: *mut Void,
) -> Long {
    // SAFETY: guaranteed by caller.
    (unsafe {
        syscall5(
            Sysno::Ipc,
            call as isize,
            first as isize,
            second as isize,
            third as isize,
            ptr.addr() as isize,
        )
    }) as Long
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::{Int, Long, UnsignedInt};

    use crate::arch::current::{Sysno, syscall5};

    use super::{MSGRCV, ipc};

    #[repr(C)]
    struct IpcKludge {
        msgp: *mut core::ffi::c_void,
        msgtyp: Long,
    }

    #[test]
    fn test_ipc_sysno() {
        assert_eq!(Sysno::Ipc as isize, 117);
    }

    #[test]
    fn test_ipc_invalid_selector_returns_einval_or_enosys() {
        // SAFETY: selector `0` is invalid, so the kernel rejects the call
        // before any pointer access.
        let rc = unsafe {
            ipc(
                0 as UnsignedInt,
                0 as Int,
                0 as Int,
                0 as Int,
                core::ptr::null_mut(),
            )
        };

        assert!(
            [-(22 as Long), -(38 as Long)].contains(&rc),
            "expected EINVAL or ENOSYS from ipc with selector 0, got {rc}",
        );
    }

    #[test]
    fn test_ipc_msgrcv_null_kludge_returns_einval_or_enosys() {
        // SAFETY: null is intentionally invalid for the MSGRCV kludge pointer
        // and is used here to exercise the entry-path rejection.
        let rc = unsafe {
            ipc(MSGRCV, 0 as Int, 0 as Int, 0 as Int, core::ptr::null_mut())
        };

        assert!(
            [-(22 as Long), -(38 as Long)].contains(&rc),
            "expected EINVAL or ENOSYS from ipc(MSGRCV, null), got {rc}",
        );
    }

    #[test]
    fn test_ipc_wrapper_matches_raw_syscall5_for_msgrcv_shape() {
        let mut msg = MaybeUninit::<u8>::uninit();
        let mut kludge = IpcKludge {
            msgp: msg.as_mut_ptr().cast(),
            msgtyp: 0x1234_5678 as Long,
        };

        let wrapped = unsafe {
            ipc(
                MSGRCV,
                0x1111_1111_u32 as Int,
                0x2222_2222_u32 as Int,
                0x3333_3333_u32 as Int,
                (&raw mut kludge).cast(),
            )
        };
        let raw = unsafe {
            syscall5(
                Sysno::Ipc,
                MSGRCV as isize,
                0x1111_1111_u32 as isize,
                0x2222_2222_u32 as isize,
                0x3333_3333_u32 as isize,
                (&raw mut kludge).addr() as isize,
            )
        } as Long;

        assert_eq!(wrapped, raw);
    }
}
