#[cfg(target_arch = "x86")]
mod x86;

pub mod current {
    #[cfg(target_arch = "x86")]
    pub use super::x86::*;
}

pub mod linux_1_0 {
    #[cfg(target_arch = "x86")]
    pub use super::x86::linux_1_0::*;
}
