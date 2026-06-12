//! OS-specific implementations behind a shared function set.
//!
//! Both `win` and `linux` must expose the same functions; the rest of the
//! codebase calls `crate::platform::*` and never touches an OS API directly.
//! See docs/linux-port.md for the porting spec.

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
