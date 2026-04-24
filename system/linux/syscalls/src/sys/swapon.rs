use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall2};

/// Enable swapping on a block device or regular file.
///
/// This wrapper spans the original Linux 1.0 x86 syscall slot `87` ABI and
/// the current native `swapon(2)` entrypoints exported by this crate on x86
/// and aarch64. Linux 1.0 took only `specialfile`, while current kernels take
/// both `specialfile` and `swap_flags`. Passing `swap_flags` remains
/// compatible with Linux 1.0 because that historical entrypoint only consumes
/// the pathname argument.
///
/// # Safety
/// - The pathname pointer must be valid to read a NUL-terminated string for
///   the duration of the syscall.
///
/// # Kernel Support
/// - Introduced: Linux 0.95
/// - Behavior changes: Linux 1.0 accepted only a pathname and therefore
///   ignores `swap_flags`; current kernels validate `swap_flags` and apply the
///   supported flag bits before activating swap.
/// - Availability: present on supported x86 and aarch64 Linux kernels
///
/// # Required Privileges
/// - Linux 1.0 requires a superuser caller.
/// - Current kernels require `CAP_SYS_ADMIN`.
///
/// # Behavior
/// - Resolves `specialfile` as a pathname.
/// - Linux 1.0 ignores `swap_flags` because its original syscall entrypoint
///   accepted only the pathname argument.
/// - Linux 1.0 accepts either a block device or a regular file.
/// - Linux 1.0 reads the first swap page and requires the legacy
///   `SWAP-SPACE` signature before enabling the swap area.
/// - Current kernels interpret `swap_flags` according to the modern
///   `swapon(2)` ABI.
///
/// # Errors
/// - `EPERM`: the caller lacks permission to activate swap.
/// - `EPERM`: Linux 1.0 has no free swap slot in its fixed
///   `MAX_SWAPFILES` array.
/// - `EINVAL`: current kernels reject unsupported `swap_flags` bits.
/// - `EFAULT`: `specialfile` is not readable as a user pathname.
/// - `ENOENT`: `specialfile` is empty, missing, or otherwise fails pathname
///   lookup with `ENOENT`.
/// - `ENAMETOOLONG`: `specialfile` exceeds one page during Linux 1.0 pathname
///   copying.
/// - `ENOMEM`: the kernel cannot allocate pathname or swap bookkeeping
///   memory.
/// - `ENOTDIR`: a path component is not a directory during lookup.
/// - `EACCES`: search permission is denied on a path component during lookup.
/// - `ELOOP`: pathname resolution encounters a symlink loop.
/// - `EBUSY`: the target inode is busy, or the block device is already active
///   as swap.
/// - `ENODEV`: Linux 1.0 resolves a block-device inode with `i_rdev == 0`.
/// - `EINVAL`: Linux 1.0 rejects non-regular, non-block targets, missing
///   swap signatures, or swap areas with no usable pages.
///
/// Filesystem-specific pathname helpers may return additional lookup errors
/// beyond the core Linux 1.0 `namei()` paths listed above.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/swapon.2.html)
/// - Stable implementation: [v7.0](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/mm/swapfile.c?h=v7.0#n3328)
/// - Stable x86 table: [v7.0 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v7.0#n102)
/// - Stable aarch64 syscall numbers:
///   [v7.0 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v7.0#n577)
/// - LTS implementation: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/mm/swapfile.c?h=v6.18.18#n3443)
/// - LTS x86 table: [v6.18.18 syscall_32.tbl](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl?h=v6.18.18#n102)
/// - LTS aarch64 syscall numbers:
///   [v6.18.18 asm-generic unistd](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/include/uapi/asm-generic/unistd.h?h=v6.18.18#n577)
/// - First stable: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/mm/swap.c?h=1.0#n726)
///
/// # Historical References
/// - First verified appearance: [Linux 0.95](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/mm/swap.c?h=0.95#n234)
pub unsafe fn swapon(specialfile: *const Char, swap_flags: Int) -> Int {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust. Linux 1.0 ignores the extra argument, while
    // current kernels interpret it as `swap_flags`.
    unsafe {
        syscall2(
            Sysno::Swapon,
            specialfile.addr() as isize,
            swap_flags as isize,
        ) as Int
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::{
        env,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::{Char, Int};

    use crate::arch::current::{Sysno, syscall2};

    use super::swapon;

    fn missing_path_bytes() -> Vec<u8> {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("celer_swapon_missing_{now}"));

        let mut bytes = path.as_os_str().as_encoded_bytes().to_vec();
        bytes.push(0);
        bytes
    }

    #[test]
    fn test_swapon_sysno() {
        #[cfg(target_arch = "x86")]
        let expected = 87;
        #[cfg(target_arch = "aarch64")]
        let expected = 224;
        #[cfg(target_arch = "x86_64")]
        let expected = 167;

        assert_eq!(Sysno::Swapon as isize, expected);
    }

    #[test]
    fn test_swapon_matches_raw_syscall_with_flags() {
        let path = missing_path_bytes();
        let ptr = path.as_ptr().cast::<Char>();
        let flags = 0x1234_5678_u32 as Int;

        // SAFETY: the pointed-to test data stays valid for the duration of the syscall.
        let wrapped = unsafe { swapon(ptr, flags) };
        // SAFETY: this uses the same raw pointer and `swap_flags` value as
        // the wrapper under test.
        let raw = unsafe {
            syscall2(Sysno::Swapon, ptr.addr() as isize, flags as isize) as Int
        };

        assert_eq!(wrapped, raw, "swapon wrapper should match raw syscall");
        assert!(wrapped < 0, "swapon unexpectedly succeeded: {wrapped}");
    }
}
