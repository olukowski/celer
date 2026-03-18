#![cfg(not(miri))] // Miri does not support syscalls
#![cfg(target_os = "linux")]
#![cfg(target_arch = "x86")]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod arch;

pub mod sys;
