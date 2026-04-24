/// A Linux errno value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Errno {
    Eacces,
    E2big,
    Ebadf,
    Efault,
    Eexist,
    Einval,
    Eintr,
    Enodev,
    Enoent,
    Enomem,
    Enosys,
    Enotdir,
    Eperm,
    Erofs,
    Raw(u16),
}

impl Errno {
    /// Converts a positive Linux errno number into an [`Errno`].
    pub const fn from_raw(raw: u16) -> Self {
        match raw {
            13 => Self::Eacces,
            7 => Self::E2big,
            9 => Self::Ebadf,
            14 => Self::Efault,
            17 => Self::Eexist,
            22 => Self::Einval,
            4 => Self::Eintr,
            19 => Self::Enodev,
            2 => Self::Enoent,
            12 => Self::Enomem,
            38 => Self::Enosys,
            20 => Self::Enotdir,
            1 => Self::Eperm,
            30 => Self::Erofs,
            other => Self::Raw(other),
        }
    }

    /// Returns `true` when `value` is in Linux's kernel errno return range.
    pub const fn is_errno(value: isize) -> bool {
        value >= -4095 && value <= -1
    }

    /// Converts a raw kernel return value into an errno when it is in the
    /// kernel errno return range.
    pub const fn from_kernel_ret(value: isize) -> Option<Self> {
        if Self::is_errno(value) {
            Some(Self::from_raw((-value) as u16))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Errno;

    #[test]
    fn test_from_raw_named_values() {
        assert_eq!(Errno::from_raw(13), Errno::Eacces);
        assert_eq!(Errno::from_raw(7), Errno::E2big);
        assert_eq!(Errno::from_raw(9), Errno::Ebadf);
        assert_eq!(Errno::from_raw(14), Errno::Efault);
        assert_eq!(Errno::from_raw(17), Errno::Eexist);
        assert_eq!(Errno::from_raw(22), Errno::Einval);
        assert_eq!(Errno::from_raw(4), Errno::Eintr);
        assert_eq!(Errno::from_raw(19), Errno::Enodev);
        assert_eq!(Errno::from_raw(2), Errno::Enoent);
        assert_eq!(Errno::from_raw(12), Errno::Enomem);
        assert_eq!(Errno::from_raw(38), Errno::Enosys);
        assert_eq!(Errno::from_raw(20), Errno::Enotdir);
        assert_eq!(Errno::from_raw(1), Errno::Eperm);
        assert_eq!(Errno::from_raw(30), Errno::Erofs);
        assert_eq!(Errno::from_raw(4095), Errno::Raw(4095));
    }

    #[test]
    fn test_is_errno_range() {
        assert!(!Errno::is_errno(0));
        assert!(Errno::is_errno(-1));
        assert!(Errno::is_errno(-4095));
        assert!(!Errno::is_errno(-4096));
    }

    #[test]
    fn test_from_kernel_ret() {
        assert_eq!(Errno::from_kernel_ret(-1), Some(Errno::Eperm));
        assert_eq!(Errno::from_kernel_ret(-4095), Some(Errno::Raw(4095)));
        assert_eq!(Errno::from_kernel_ret(0), None);
        assert_eq!(Errno::from_kernel_ret(-4096), None);
    }
}
