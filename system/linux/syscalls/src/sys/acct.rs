use celer_system_linux_ctypes::{Char, Int};

use crate::arch::current::{Sysno, syscall1};

/// Enable or disable process accounting for the current PID namespace.
///
/// # Kernel Support
/// - Introduced: Linux 1.0
/// - Behavior changes: none known
/// - Availability: always present on supported Linux kernels
///
/// # Required Privileges
/// - `CAP_SYS_PACCT`
///
/// # Behavior
/// - A null `name` disables accounting for the current PID namespace.
/// - A non-null `name` enables accounting on the named file.
/// - The kernel rejects callers without `CAP_SYS_PACCT` before consulting the
///   path.
///
/// # Errors
/// - `EPERM`: the caller lacks `CAP_SYS_PACCT`.
/// - Additional errno values may come from the kernel's path and file handling
///   helpers when `name` is non-null.
///
/// # References
/// - `man` [page](https://man7.org/linux/man-pages/man2/acct.2.html)
/// - Stable: [v6.19](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/acct.c?h=v6.19#n293)
/// - LTS: [v6.18.18](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/kernel/acct.c?h=v6.18.18#n293)
///
/// # Historical References
/// - First appearance: [Linux 1.0](https://git.kernel.org/pub/scm/linux/kernel/git/history/history.git/tree/kernel/sys.c?h=1.0#n294)
pub fn acct(name: *const Char) -> Int {
    // SAFETY: this wrapper forwards the raw pathname pointer without
    // dereferencing it in Rust, so invalid pointers are reported by the
    // kernel as syscall errors rather than causing Rust UB.
    unsafe { syscall1(Sysno::Acct, name.addr() as isize) as Int }
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        time::{SystemTime, UNIX_EPOCH},
    };

    use celer_system_linux_ctypes::Char;

    use super::acct;

    fn create_temp_path() -> Vec<u8> {
        let mut path = env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("test_acct_{now}"));

        let mut bytes = path.as_os_str().as_encoded_bytes().to_vec();
        bytes.push(0);
        bytes
    }

    #[test]
    fn test_acct_invalid_path() {
        let path = create_temp_path();

        let result = acct(path.as_ptr().cast::<Char>());

        assert!(result < 0, "acct should have failed: {result}");
    }
}
