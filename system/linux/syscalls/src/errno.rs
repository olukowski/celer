/// A Linux errno value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Errno {
    Eacces,
    Eexist,
    Einval,
    Enoent,
    Enomem,
    Eperm,
    Erofs,
    Raw(u16),
}

impl Errno {
    fn from_raw(raw: u16) -> Self {
        match raw {
            13 => Self::Eacces,
            17 => Self::Eexist,
            22 => Self::Einval,
            2 => Self::Enoent,
            12 => Self::Enomem,
            1 => Self::Eperm,
            30 => Self::Erofs,
            other => Self::Raw(other),
        }
    }

    /// Returns `true` when `value` is in Linux's kernel errno return range.
    pub const fn is_errno(value: isize) -> bool {
        (-4095..=-1).contains(&value)
    }
}

impl From<isize> for Errno {
    fn from(value: isize) -> Self {
        if Self::is_errno(value) {
            Self::from_raw((-value) as u16)
        } else {
            Self::Raw(value as u16)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Errno;

    #[test]
    fn test_is_errno_range() {
        assert!(!Errno::is_errno(0));
        assert!(Errno::is_errno(-1));
        assert!(Errno::is_errno(-4095));
        assert!(!Errno::is_errno(-4096));
    }
}
