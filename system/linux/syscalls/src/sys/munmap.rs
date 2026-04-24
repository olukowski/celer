use celer_system_linux_ctypes::{Int, SizeT, UnsignedLong};

use crate::arch::current::{Sysno, syscall2};

/// Unmap a range from the calling process's virtual address space.
///
/// # Safety
/// - The caller must ensure the unmapped address range is not still relied on
///   by any live Rust references, slices, or raw pointers after [`munmap`]
///   returns.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: Linux 1.0 treats `len == 0` as a no-op success; current
///   kernels reject `len == 0` with `EINVAL`
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - None
///
/// # Behavior
/// - Linux 1.0 validates that `addr` is page-aligned and that the requested
///   range does not exceed `TASK_SIZE` before doing any unmap work.
/// - Linux 1.0 rounds `len` up to a page boundary before unmapping.
/// - Linux 1.0 returns success without changing anything if `len == 0`.
/// - Linux 1.0 also returns success if the requested range overlaps no mapping.
/// - Current stable and LTS kernels first strip any address tag bits from
///   `addr`.
/// - Current stable and LTS kernels still round `len` up to a page boundary
///   and still return success if the requested range overlaps no mapping.
/// - Current stable and LTS kernels reject `len == 0` with `EINVAL`.
///
/// # Errors
/// - `EINVAL`: in Linux 1.0, `addr` is not page-aligned or the requested range
///   exceeds `TASK_SIZE`.
/// - `EINVAL`: in current stable and LTS kernels, `addr` is not page-aligned,
///   `addr` exceeds `TASK_SIZE`, `len` exceeds `TASK_SIZE - addr`, or
///   `len == 0`.
///
/// Current stable and LTS kernels can also return `EINTR`, `ENOMEM`, or
/// `EPERM` from their VMA teardown path.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/munmap.2.html)
/// - Stable: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/mm/mmap.c?h=v7.0#n1075)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/mm/mmap.c?h=v6.18.18#n1077)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/mm/mmap.c?h=1.0#n235)
///
/// # Historical References
/// - Linux 1.0 `do_munmap` validation:
///   [mm/mmap.c](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/mm/mmap.c?h=1.0#n246)
pub unsafe fn munmap(addr: UnsignedLong, len: SizeT) -> Int {
    // SAFETY: the caller must uphold the aliasing and lifetime requirements of
    // removing this address range from the current process.
    unsafe { syscall2(Sysno::Munmap, addr as isize, len as isize) as Int }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::ptr;

    use celer_system_linux_ctypes::{
        Long, SizeT, UnsignedInt, UnsignedLong, Void,
    };

    use crate::arch::current::Sysno;

    use super::munmap;
    use crate::sys::mmap;

    const PROT_READ: UnsignedLong = 0x1;
    const PROT_WRITE: UnsignedLong = 0x2;
    const MAP_PRIVATE: UnsignedLong = 0x02;
    const MAP_ANONYMOUS: UnsignedLong = 0x20;

    fn is_errno_result(raw: UnsignedLong) -> bool {
        raw >= ((-4095 as Long) as UnsignedLong)
    }

    fn page_size() -> usize {
        4096
    }

    #[test]
    fn test_munmap_syscall_number() {
        #[cfg(target_arch = "x86")]
        let expected = 91;
        #[cfg(target_arch = "aarch64")]
        let expected = 215;
        #[cfg(target_arch = "x86_64")]
        let expected = 11;

        assert_eq!(Sysno::Munmap as isize, expected);
    }

    #[test]
    fn test_munmap_success_and_repeat_noop() {
        let len = page_size();
        let mapping = unsafe {
            mmap(
                ptr::null_mut(),
                len as SizeT,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                0 as UnsignedInt,
                0,
            )
        };

        assert!(
            !is_errno_result(mapping),
            "mmap failed: {}",
            mapping as Long
        );

        let addr = ptr::without_provenance_mut::<Void>(mapping as usize);

        // SAFETY: this test stops using the mapping after unmapping it.
        let rc = unsafe { munmap(addr.addr() as UnsignedLong, len as SizeT) };
        assert_eq!(rc, 0, "munmap failed: {rc}");

        // SAFETY: the range is already unmapped, so Linux should treat this as
        // a successful no-op.
        let second =
            unsafe { munmap(addr.addr() as UnsignedLong, len as SizeT) };
        assert_eq!(second, 0, "second munmap should be a no-op: {second}");
    }

    #[test]
    fn test_munmap_rounds_len_up_to_page_size() {
        let len = page_size();
        let mapping = unsafe {
            mmap(
                ptr::null_mut(),
                len as SizeT,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                0 as UnsignedInt,
                0,
            )
        };

        assert!(
            !is_errno_result(mapping),
            "mmap failed: {}",
            mapping as Long
        );

        let addr = ptr::without_provenance_mut::<Void>(mapping as usize);

        // SAFETY: this test stops using the mapping after unmapping it.
        let rc = unsafe { munmap(addr.addr() as UnsignedLong, 1 as SizeT) };
        assert_eq!(rc, 0, "munmap failed: {rc}");

        // SAFETY: if the kernel rounded `len` up to a full page, this second
        // call is a successful no-op.
        let second =
            unsafe { munmap(addr.addr() as UnsignedLong, len as SizeT) };
        assert_eq!(second, 0, "rounded munmap should have removed the page");
    }

    #[test]
    fn test_munmap_unaligned_addr_returns_einval() {
        let rc = unsafe { munmap(1 as UnsignedLong, page_size() as SizeT) };

        assert_eq!(rc, -22, "expected EINVAL for unaligned addr, got {rc}");
    }

    #[test]
    fn test_munmap_out_of_range_addr_returns_einval() {
        let rc = unsafe { munmap(UnsignedLong::MAX, page_size() as SizeT) };

        assert_eq!(rc, -22, "expected EINVAL for out-of-range addr, got {rc}");
    }

    #[test]
    fn test_munmap_zero_len_is_currently_einval() {
        let rc = unsafe { munmap(0 as UnsignedLong, 0 as SizeT) };

        assert_eq!(rc, -22, "expected EINVAL on current kernels, got {rc}");
    }
}
