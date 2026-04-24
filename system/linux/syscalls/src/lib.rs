#![cfg(not(miri))] // Miri does not support syscalls
#![cfg(target_os = "linux")]
#![cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "aarch64"
))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod arch;

pub mod sys;
