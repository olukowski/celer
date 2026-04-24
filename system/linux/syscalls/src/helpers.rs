use crate::errno::Errno;

pub(crate) fn result_from_ret<T, E>(
    ret: isize,
    success: impl FnOnce(isize) -> T,
    error: impl FnOnce(Errno) -> E,
) -> Result<T, E> {
    Errno::from_kernel_ret(ret)
        .map(error)
        .map_or_else(|| Ok(success(ret)), Err)
}

pub(crate) fn unit_from_ret<E>(
    ret: isize,
    error: impl FnOnce(Errno) -> E,
) -> Result<(), E> {
    result_from_ret(ret, |_| (), error)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{result_from_ret, unit_from_ret};
    use crate::errno::Errno;

    #[test]
    fn test_result_from_ret_success() {
        assert_eq!(
            result_from_ret(7, |ret| ret + 1, |errno| errno),
            Ok::<_, Errno>(8)
        );
    }

    #[test]
    fn test_result_from_ret_error() {
        assert_eq!(
            result_from_ret(-22, |ret| ret, |errno| errno),
            Err(Errno::Einval)
        );
    }

    #[test]
    fn test_unit_from_ret_success() {
        assert_eq!(unit_from_ret(0, |errno| errno), Ok::<_, Errno>(()));
    }

    #[test]
    fn test_unit_from_ret_error() {
        assert_eq!(unit_from_ret(-1, |errno| errno), Err(Errno::Eperm));
    }
}
