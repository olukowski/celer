use core::mem::MaybeUninit;

use celer_system_linux_ctypes::{Int, KernelSym};

use crate::errno::Errno;
use crate::helpers::result_from_ret;
use crate::sys;

/// Errors returned by [`get_kernel_syms`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GetKernelSymsError {
    /// `EFAULT`.
    Efault,
    /// `ENOSYS`.
    Enosys,
    /// Another errno returned by a historical kernel implementation.
    Other(Errno),
}

impl GetKernelSymsError {
    fn from_errno(errno: Errno) -> Self {
        match errno {
            Errno::Efault => Self::Efault,
            Errno::Enosys => Self::Enosys,
            errno => Self::Other(errno),
        }
    }
}

fn table_ptr(table: Option<&mut [MaybeUninit<KernelSym>]>) -> *mut KernelSym {
    match table {
        Some(table) => table.as_mut_ptr().cast::<KernelSym>(),
        None => core::ptr::null_mut(),
    }
}

fn get_kernel_syms_from_ret(ret: Int) -> Result<usize, GetKernelSymsError> {
    result_from_ret(
        ret as isize,
        |ret| ret as usize,
        GetKernelSymsError::from_errno,
    )
}

/// Copy the historical Linux 1.0 kernel symbol table into `table`, or query
/// the symbol count by passing `None`.
///
/// This wrapper keeps the raw kernel ABI shape but replaces the output pointer
/// with an optional mutable slice of `MaybeUninit<KernelSym>`.
///
/// The returned `usize` is the kernel's symbol count.
///
/// See [`sys::get_kernel_syms`] for kernel behavior, reachable errors, and
/// source references.
///
/// # Safety
/// - If `table` is `Some`, it must provide writable space for at least the
///   kernel's symbol count.
///
/// # Errors
/// - [`GetKernelSymsError::Efault`]: the kernel could not access the output
///   table.
/// - [`GetKernelSymsError::Enosys`]: the syscall slot is unimplemented on the
///   running kernel.
/// - [`GetKernelSymsError::Other`]: another historical kernel errno.
pub unsafe fn get_kernel_syms(
    table: Option<&mut [MaybeUninit<KernelSym>]>,
) -> Result<usize, GetKernelSymsError> {
    let ptr = table_ptr(table);

    // SAFETY: the caller upholds any buffer-capacity precondition when `table`
    // is provided.
    let ret = unsafe { sys::get_kernel_syms(ptr) };
    get_kernel_syms_from_ret(ret)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::mem::MaybeUninit;

    use celer_system_linux_ctypes::KernelSym;

    use crate::{errno::Errno, sys};

    use super::{
        GetKernelSymsError, get_kernel_syms, get_kernel_syms_from_ret,
        table_ptr,
    };

    #[test]
    fn test_get_kernel_syms_null_matches_raw() {
        let wrapped = unsafe { get_kernel_syms(None) };
        let raw = unsafe { sys::get_kernel_syms(core::ptr::null_mut()) };

        match raw {
            r if r >= 0 => assert_eq!(wrapped, Ok(r as usize)),
            r => assert_eq!(
                wrapped,
                Err(GetKernelSymsError::from_errno(
                    Errno::from_kernel_ret(r as isize).unwrap(),
                ))
            ),
        }
    }

    #[test]
    fn test_get_kernel_syms_error_mapping() {
        assert_eq!(
            GetKernelSymsError::from_errno(Errno::Efault),
            GetKernelSymsError::Efault
        );
        assert_eq!(
            GetKernelSymsError::from_errno(Errno::Enosys),
            GetKernelSymsError::Enosys
        );
        assert_eq!(
            GetKernelSymsError::from_errno(Errno::Enomem),
            GetKernelSymsError::Other(Errno::Enomem)
        );
    }

    #[test]
    fn test_get_kernel_syms_success_mapping() {
        assert_eq!(get_kernel_syms_from_ret(7), Ok(7));
    }

    #[test]
    fn test_get_kernel_syms_table_ptr() {
        assert!(table_ptr(None).is_null());

        let mut table = [MaybeUninit::<KernelSym>::uninit()];
        assert_eq!(table_ptr(Some(&mut table)), table.as_mut_ptr().cast());
    }
}
