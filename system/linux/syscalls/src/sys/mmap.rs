use celer_system_linux_ctypes::{SizeT, UnsignedInt, UnsignedLong, Void};

use crate::arch::current::Sysno;
#[cfg(target_arch = "x86")]
use crate::arch::current::syscall1;
#[cfg(target_arch = "aarch64")]
use crate::arch::current::syscall6;

#[cfg(target_arch = "x86")]
#[repr(C)]
struct MmapArgs {
    addr: UnsignedLong,
    len: UnsignedLong,
    prot: UnsignedLong,
    flags: UnsignedLong,
    fd: UnsignedLong,
    offset: UnsignedLong,
}

/// Create a memory mapping.
///
/// This wrapper targets the original Linux 1.0 i386 syscall slot 90 ABI.
/// That historical entry takes a single pointer to six packed words; this
/// wrapper builds the packed block internally and exposes the logical
/// `mmap(addr, len, prot, flags, fd, offset)` interface.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 forwards `offset` unchanged from the syscall
///   entry to the selected mapper and returns `addr` unchanged when `len`
///   rounds down to zero; current x86 kernels keep syscall slot 90 as the
///   legacy `old_mmap` ABI, but they reject non-page-aligned `offset` values
///   at the syscall entry and reject zero effective length
/// - Availability: present on supported x86 Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Safety
/// - The caller must ensure the requested mapping operation does not invalidate
///   any live Rust references, raw pointers whose pointees must remain valid,
///   or allocator assumptions in the current process.
/// - In particular, using `MAP_FIXED` or otherwise replacing an existing
///   mapping can make previously valid process memory inaccessible or reused
///   for unrelated data.
///
/// # Behavior
/// - `addr` is a hint unless `flags` includes `MAP_FIXED`.
/// - Linux 1.0 reads the six arguments from a temporary packed block in this
///   order: `addr`, `len`, `prot`, `flags`, `fd`, `offset`.
/// - If `flags` includes `MAP_ANONYMOUS`, Linux 1.0 ignores `fd`.
/// - On Linux 1.0, if `len` rounds up to zero pages, the syscall succeeds and
///   returns `addr` unchanged.
/// - On Linux 1.0, non-fixed mappings search only the
///   `0x40000000..0x60000000` shared-memory window.
/// - On Linux 1.0, the kernel removes any existing mappings in the chosen
///   range before attempting the new mapping and does not roll them back if
///   the mapper then fails.
/// - The raw return value is address-valued. Callers that want to interpret
///   errors should cast the return value to
///   [`Long`](celer_system_linux_ctypes::Long) or `isize` before checking for
///   negative errno results.
///
/// # Errors
/// - `EACCES`: a file-backed shared writable mapping lacks write permission,
///   or a file-backed private mapping lacks read permission.
/// - `EBADF`: `MAP_ANONYMOUS` is clear and `fd` does not refer to an open
///   file descriptor.
/// - `EINVAL`: `addr` and `len` describe an invalid range, `MAP_FIXED` uses a
///   non-page-aligned `addr`, `flags & MAP_TYPE` is neither `MAP_SHARED` nor
///   `MAP_PRIVATE`, or `prot` produces no effective access mask.
/// - `ENODEV`: the target file object has no `mmap` file operation.
/// - `ENOMEM`: Linux 1.0 found no free non-fixed hole in its shared-memory
///   search window, or the selected mapper could not allocate required kernel
///   memory.
/// - `ENOEXEC`: common Linux 1.0 filesystem `generic_mmap` paths reject the
///   target inode because it does not supply `bmap`.
///
/// Filesystem-specific `mmap` handlers may return additional reachable errors.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/mmap.2.html)
/// - Stable i386 entry:
///   [v7.0 ia32_mmap](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/kernel/sys_ia32.c?h=v7.0#n223)
/// - Stable shared implementation:
///   [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/mm/mmap.c?h=v6.19#n621)
/// - LTS i386 entry:
///   [v6.18.18 ia32_mmap](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/kernel/sys_ia32.c?h=v6.18.18#n223)
/// - LTS shared implementation:
///   [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/mm/mmap.c?h=v6.18.18#n621)
/// - First stable:
///   [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/mm/mmap.c?h=1.0#n137)
/// - Linux 1.0 syscall table:
///   [include/linux/unistd.h](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/include/linux/unistd.h?h=1.0#n99)
pub unsafe fn mmap(
    addr: *mut Void,
    len: SizeT,
    prot: UnsignedLong,
    flags: UnsignedLong,
    fd: UnsignedInt,
    offset: UnsignedLong,
) -> UnsignedLong {
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: the caller upholds the process-memory invariants required
        // by this mapping operation.
        unsafe {
            syscall6(
                Sysno::Mmap,
                addr.addr() as isize,
                len as isize,
                prot as isize,
                flags as isize,
                fd as isize,
                offset as isize,
            ) as UnsignedLong
        }
    }

    #[cfg(target_arch = "x86")]
    {
        let args = MmapArgs {
            addr: addr.addr() as UnsignedLong,
            len: len as UnsignedLong,
            prot,
            flags,
            fd: fd as UnsignedLong,
            offset,
        };

        // SAFETY: the caller upholds the process-memory invariants required by
        // this mapping operation, and `args` is a valid six-word ABI block.
        unsafe {
            syscall1(Sysno::Mmap, (&raw const args).addr() as isize)
                as UnsignedLong
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::ptr;

    use celer_system_linux_ctypes::{Int, Long, SizeT, UnsignedLong, Void};

    use crate::arch::current::{Sysno, syscall1};

    use super::{MmapArgs, mmap};

    unsafe extern "C" {
        fn munmap(addr: *mut Void, len: SizeT) -> Int;
    }

    const PROT_NONE: UnsignedLong = 0;
    const PROT_READ: UnsignedLong = 0x1;
    const PROT_WRITE: UnsignedLong = 0x2;
    const MAP_PRIVATE: UnsignedLong = 0x2;
    const MAP_ANONYMOUS: UnsignedLong = 0x20;

    fn is_errno_result(raw: UnsignedLong) -> bool {
        raw >= ((-4095 as Long) as UnsignedLong)
    }

    unsafe fn raw_old_mmap(args: *const MmapArgs) -> UnsignedLong {
        // SAFETY: the caller provides a readable six-word `old_mmap` arg block.
        unsafe { syscall1(Sysno::Mmap, args.addr() as isize) as UnsignedLong }
    }

    #[test]
    fn test_mmap_sysno() {
        assert_eq!(Sysno::Mmap as isize, 90);
    }

    #[test]
    fn test_mmap_args_layout() {
        assert_eq!(core::mem::size_of::<MmapArgs>(), 24);
        assert_eq!(core::mem::align_of::<MmapArgs>(), 4);
        assert_eq!(core::mem::offset_of!(MmapArgs, addr), 0);
        assert_eq!(core::mem::offset_of!(MmapArgs, len), 4);
        assert_eq!(core::mem::offset_of!(MmapArgs, prot), 8);
        assert_eq!(core::mem::offset_of!(MmapArgs, flags), 12);
        assert_eq!(core::mem::offset_of!(MmapArgs, fd), 16);
        assert_eq!(core::mem::offset_of!(MmapArgs, offset), 20);
    }

    #[test]
    fn test_mmap_matches_raw_syscall_for_invalid_fd() {
        let args = MmapArgs {
            addr: 0,
            len: 4096,
            prot: PROT_READ,
            flags: MAP_PRIVATE,
            fd: 9_999,
            offset: 0,
        };

        // SAFETY: this uses the same logical arguments as the raw syscall.
        let wrapped = unsafe {
            mmap(ptr::null_mut(), 4096, PROT_READ, MAP_PRIVATE, 9_999, 0)
        };
        // SAFETY: `args` is a readable six-word `old_mmap` block.
        let raw = unsafe { raw_old_mmap(&raw const args) };

        assert_eq!(wrapped as Long, raw as Long);
        assert_eq!(wrapped as Long, -(9 as Long));
    }

    #[test]
    fn test_mmap_rejects_unaligned_offset_on_current_kernel() {
        // SAFETY: this request does not target existing mappings and uses the
        // kernel-managed placement path.
        let rc = unsafe {
            mmap(
                ptr::null_mut(),
                4096,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
                0,
                1,
            )
        };

        assert_eq!(rc as Long, -(22 as Long));
    }

    #[test]
    fn test_mmap_anonymous_round_trip() {
        // SAFETY: this request lets the kernel choose the mapping address and
        // does not replace any existing mapping.
        let mapped = unsafe {
            mmap(
                ptr::null_mut(),
                4096,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                0,
                0,
            )
        };

        assert!(
            !is_errno_result(mapped),
            "mmap unexpectedly failed: {}",
            mapped as Long
        );

        let ptr = ptr::without_provenance_mut::<u8>(mapped as usize);
        // SAFETY: the successful mapping covers at least one writable byte.
        unsafe {
            ptr.write(0x5A);
            assert_eq!(ptr.read(), 0x5A);
            assert_eq!(munmap(ptr.cast(), 4096), 0);
        }
    }

    #[test]
    fn test_mmap_prot_none_matches_raw_syscall_shape() {
        let args = MmapArgs {
            addr: 0,
            len: 4096,
            prot: PROT_NONE,
            flags: MAP_PRIVATE | MAP_ANONYMOUS,
            fd: 0,
            offset: 0,
        };

        // SAFETY: this request lets the kernel choose the mapping address.
        let wrapped = unsafe {
            mmap(
                ptr::null_mut(),
                4096,
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                0,
                0,
            )
        };
        // SAFETY: `args` is a readable six-word `old_mmap` block.
        let raw = unsafe { raw_old_mmap(&raw const args) };

        assert_eq!(is_errno_result(wrapped), is_errno_result(raw));

        if is_errno_result(wrapped) {
            assert_eq!(wrapped as Long, raw as Long);
            return;
        }

        assert_eq!(
            wrapped & 0xfff,
            0,
            "wrapped mapping should be page-aligned"
        );
        assert_eq!(raw & 0xfff, 0, "raw mapping should be page-aligned");

        // SAFETY: both successful mappings refer to one page that this test no
        // longer uses after unmapping.
        unsafe {
            assert_eq!(
                munmap(
                    ptr::without_provenance_mut::<Void>(wrapped as usize),
                    4096
                ),
                0
            );
            assert_eq!(
                munmap(ptr::without_provenance_mut::<Void>(raw as usize), 4096),
                0
            );
        }
    }
}
