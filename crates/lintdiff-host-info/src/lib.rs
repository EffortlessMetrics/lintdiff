//! Host information detection for lintdiff.
//!
//! Provides types and functions for detecting host operating system,
//! CPU architecture, and hostname information.
//!
//! # Example
//!
//! ```
//! use lintdiff_host_info::{detect_host, format_host_info, format_rust_target, format_oci_platform};
//!
//! let info = detect_host();
//! println!("Host: {}", format_host_info(&info));
//! println!("Rust target: {}", format_rust_target(&info));
//! println!("OCI platform: {}", format_oci_platform(&info));
//! ```

use std::env;

/// Operating system types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OsType {
    /// Microsoft Windows
    Windows,
    /// Linux
    Linux,
    /// Apple macOS
    MacOS,
    /// FreeBSD
    FreeBSD,
    /// Other/unknown operating system
    Other,
}

impl OsType {
    /// Returns the string representation of the OS type.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_host_info::OsType;
    ///
    /// assert_eq!(OsType::Windows.as_str(), "windows");
    /// assert_eq!(OsType::Linux.as_str(), "linux");
    /// assert_eq!(OsType::MacOS.as_str(), "macos");
    /// assert_eq!(OsType::FreeBSD.as_str(), "freebsd");
    /// assert_eq!(OsType::Other.as_str(), "other");
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::MacOS => "macos",
            Self::FreeBSD => "freebsd",
            Self::Other => "other",
        }
    }

    /// Detects the current operating system.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_host_info::OsType;
    ///
    /// let os = OsType::detect();
    /// println!("Detected OS: {}", os.as_str());
    /// ```
    #[must_use]
    pub fn detect() -> Self {
        match env::consts::OS {
            "windows" => Self::Windows,
            "linux" => Self::Linux,
            "macos" => Self::MacOS,
            "freebsd" => Self::FreeBSD,
            _ => Self::Other,
        }
    }
}

impl Default for OsType {
    fn default() -> Self {
        Self::detect()
    }
}

impl std::fmt::Display for OsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// CPU architecture types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ArchType {
    /// 32-bit x86 (`i386`, `i686`)
    X86,
    /// 64-bit x86 (`x86_64`, AMD64)
    X64,
    /// 32-bit ARM
    Arm,
    /// 64-bit ARM (`AArch64`)
    Arm64,
    /// Other/unknown architecture
    Other,
}

impl ArchType {
    /// Returns the string representation of the architecture type.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_host_info::ArchType;
    ///
    /// assert_eq!(ArchType::X86.as_str(), "x86");
    /// assert_eq!(ArchType::X64.as_str(), "x64");
    /// assert_eq!(ArchType::Arm.as_str(), "arm");
    /// assert_eq!(ArchType::Arm64.as_str(), "arm64");
    /// assert_eq!(ArchType::Other.as_str(), "other");
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::X86 => "x86",
            Self::X64 => "x64",
            Self::Arm => "arm",
            Self::Arm64 => "arm64",
            Self::Other => "other",
        }
    }

    /// Detects the current CPU architecture.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_host_info::ArchType;
    ///
    /// let arch = ArchType::detect();
    /// println!("Detected architecture: {}", arch.as_str());
    /// ```
    #[must_use]
    pub fn detect() -> Self {
        match env::consts::ARCH {
            "x86" | "i386" | "i686" => Self::X86,
            "x86_64" | "x64" | "amd64" => Self::X64,
            "arm" => Self::Arm,
            "aarch64" | "arm64" => Self::Arm64,
            _ => Self::Other,
        }
    }
}

impl Default for ArchType {
    fn default() -> Self {
        Self::detect()
    }
}

impl std::fmt::Display for ArchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Host information containing OS, architecture, and optional hostname.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HostInfo {
    /// Operating system type
    pub os: OsType,
    /// CPU architecture type
    pub arch: ArchType,
    /// Hostname if available
    pub hostname: Option<String>,
}

impl HostInfo {
    /// Creates a new `HostInfo` with the given values.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_host_info::{HostInfo, OsType, ArchType};
    ///
    /// let info = HostInfo::new(OsType::Linux, ArchType::X64, Some("myhost".to_string()));
    /// assert_eq!(info.os, OsType::Linux);
    /// assert_eq!(info.arch, ArchType::X64);
    /// assert_eq!(info.hostname, Some("myhost".to_string()));
    /// ```
    #[must_use]
    pub const fn new(os: OsType, arch: ArchType, hostname: Option<String>) -> Self {
        Self { os, arch, hostname }
    }

    /// Creates a `HostInfo` without a hostname.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_host_info::{HostInfo, OsType, ArchType};
    ///
    /// let info = HostInfo::without_hostname(OsType::Windows, ArchType::X64);
    /// assert_eq!(info.os, OsType::Windows);
    /// assert_eq!(info.hostname, None);
    /// ```
    #[must_use]
    pub const fn without_hostname(os: OsType, arch: ArchType) -> Self {
        Self {
            os,
            arch,
            hostname: None,
        }
    }

    /// Detects the current host information.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_host_info::HostInfo;
    ///
    /// let info = HostInfo::detect();
    /// println!("OS: {}, Arch: {}", info.os.as_str(), info.arch.as_str());
    /// ```
    #[must_use]
    pub fn detect() -> Self {
        Self {
            os: OsType::detect(),
            arch: ArchType::detect(),
            hostname: get_hostname(),
        }
    }
}

impl Default for HostInfo {
    fn default() -> Self {
        Self::detect()
    }
}

/// Detects the current host information.
///
/// This is a convenience function that calls [`HostInfo::detect`].
///
/// # Examples
///
/// ```
/// use lintdiff_host_info::detect_host;
///
/// let info = detect_host();
/// println!("OS: {}, Arch: {}", info.os.as_str(), info.arch.as_str());
/// ```
#[must_use]
pub fn detect_host() -> HostInfo {
    HostInfo::detect()
}

/// Detects the current operating system.
///
/// This is a convenience function that calls [`OsType::detect`].
///
/// # Examples
///
/// ```
/// use lintdiff_host_info::detect_os;
///
/// let os = detect_os();
/// println!("Detected OS: {}", os.as_str());
/// ```
#[must_use]
pub fn detect_os() -> OsType {
    OsType::detect()
}

/// Detects the current CPU architecture.
///
/// This is a convenience function that calls [`ArchType::detect`].
///
/// # Examples
///
/// ```
/// use lintdiff_host_info::detect_arch;
///
/// let arch = detect_arch();
/// println!("Detected architecture: {}", arch.as_str());
/// ```
#[must_use]
pub fn detect_arch() -> ArchType {
    ArchType::detect()
}

/// Gets the hostname of the current machine.
///
/// Returns `None` if the hostname cannot be determined.
///
/// # Examples
///
/// ```
/// use lintdiff_host_info::get_hostname;
///
/// if let Some(hostname) = get_hostname() {
///     println!("Hostname: {}", hostname);
/// } else {
///     println!("Hostname not available");
/// }
/// ```
#[must_use]
pub fn get_hostname() -> Option<String> {
    hostname::get()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s: &String| !s.is_empty())
}

/// Formats host information in a human-readable format.
///
/// # Examples
///
/// ```
/// use lintdiff_host_info::{HostInfo, OsType, ArchType, format_host_info};
///
/// let info = HostInfo::new(OsType::Linux, ArchType::X64, Some("myhost".to_string()));
/// let formatted = format_host_info(&info);
/// assert_eq!(formatted, "myhost (linux/x64)");
/// ```
#[must_use]
pub fn format_host_info(info: &HostInfo) -> String {
    info.hostname.as_ref().map_or_else(
        || format!("{}/{}", info.os.as_str(), info.arch.as_str()),
        |hostname| format!("{} ({}/{})", hostname, info.os.as_str(), info.arch.as_str()),
    )
}

/// Formats host information as a Rust target triple.
///
/// Returns a best-effort target triple based on the OS and architecture.
/// For unknown combinations, falls back to a generic format.
///
/// # Examples
///
/// ```
/// use lintdiff_host_info::{HostInfo, OsType, ArchType, format_rust_target};
///
/// let info = HostInfo::without_hostname(OsType::Windows, ArchType::X64);
/// let target = format_rust_target(&info);
/// assert_eq!(target, "x86_64-pc-windows-msvc");
/// ```
#[must_use]
pub fn format_rust_target(info: &HostInfo) -> String {
    match (info.os, info.arch) {
        // Windows targets
        (OsType::Windows, ArchType::X64) => "x86_64-pc-windows-msvc".to_string(),
        (OsType::Windows, ArchType::X86) => "i686-pc-windows-msvc".to_string(),
        (OsType::Windows, ArchType::Arm64) => "aarch64-pc-windows-msvc".to_string(),
        // Linux targets
        (OsType::Linux, ArchType::X64) => "x86_64-unknown-linux-gnu".to_string(),
        (OsType::Linux, ArchType::X86) => "i686-unknown-linux-gnu".to_string(),
        (OsType::Linux, ArchType::Arm) => "arm-unknown-linux-gnueabihf".to_string(),
        (OsType::Linux, ArchType::Arm64) => "aarch64-unknown-linux-gnu".to_string(),
        // macOS targets
        (OsType::MacOS, ArchType::X64) => "x86_64-apple-darwin".to_string(),
        (OsType::MacOS, ArchType::Arm64) => "aarch64-apple-darwin".to_string(),
        // FreeBSD targets
        (OsType::FreeBSD, ArchType::X64) => "x86_64-unknown-freebsd".to_string(),
        (OsType::FreeBSD, ArchType::X86) => "i686-unknown-freebsd".to_string(),
        // Fallback for unknown combinations
        (os, arch) => format!("{}-unknown-{}", arch_str_for_target(arch), os.as_str()),
    }
}

/// Formats host information as an OCI platform string.
///
/// Returns a platform string in the format `os/arch` (e.g., `linux/amd64`).
///
/// # Examples
///
/// ```
/// use lintdiff_host_info::{HostInfo, OsType, ArchType, format_oci_platform};
///
/// let info = HostInfo::without_hostname(OsType::Linux, ArchType::X64);
/// let platform = format_oci_platform(&info);
/// assert_eq!(platform, "linux/amd64");
/// ```
#[must_use]
pub fn format_oci_platform(info: &HostInfo) -> String {
    format!("{}/{}", info.os.as_str(), arch_to_oci(info.arch))
}

/// Converts `ArchType` to the architecture string used in Rust targets.
const fn arch_str_for_target(arch: ArchType) -> &'static str {
    match arch {
        ArchType::X86 => "i686",
        ArchType::X64 => "x86_64",
        ArchType::Arm => "arm",
        ArchType::Arm64 => "aarch64",
        ArchType::Other => "unknown",
    }
}

/// Converts `ArchType` to OCI platform architecture naming.
const fn arch_to_oci(arch: ArchType) -> &'static str {
    match arch {
        ArchType::X86 => "386",
        ArchType::X64 => "amd64",
        ArchType::Arm => "arm",
        ArchType::Arm64 => "arm64",
        ArchType::Other => "unknown",
    }
}

// Internal hostname module to avoid external dependencies
mod hostname {
    use std::env;

    /// Gets the hostname from environment variables or system info.
    #[allow(unreachable_pub)]
    pub fn get() -> Option<std::ffi::OsString> {
        // Try common hostname environment variables
        if let Ok(hostname) = env::var("HOSTNAME") {
            if !hostname.is_empty() {
                return Some(hostname.into());
            }
        }
        if let Ok(hostname) = env::var("COMPUTERNAME") {
            if !hostname.is_empty() {
                return Some(hostname.into());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_type_as_str() {
        assert_eq!(OsType::Windows.as_str(), "windows");
        assert_eq!(OsType::Linux.as_str(), "linux");
        assert_eq!(OsType::MacOS.as_str(), "macos");
        assert_eq!(OsType::FreeBSD.as_str(), "freebsd");
        assert_eq!(OsType::Other.as_str(), "other");
    }

    #[test]
    fn test_arch_type_as_str() {
        assert_eq!(ArchType::X86.as_str(), "x86");
        assert_eq!(ArchType::X64.as_str(), "x64");
        assert_eq!(ArchType::Arm.as_str(), "arm");
        assert_eq!(ArchType::Arm64.as_str(), "arm64");
        assert_eq!(ArchType::Other.as_str(), "other");
    }

    #[test]
    fn test_host_info_new() {
        let info = HostInfo::new(OsType::Linux, ArchType::X64, Some("testhost".to_string()));
        assert_eq!(info.os, OsType::Linux);
        assert_eq!(info.arch, ArchType::X64);
        assert_eq!(info.hostname, Some("testhost".to_string()));
    }

    #[test]
    fn test_host_info_without_hostname() {
        let info = HostInfo::without_hostname(OsType::Windows, ArchType::Arm64);
        assert_eq!(info.os, OsType::Windows);
        assert_eq!(info.arch, ArchType::Arm64);
        assert_eq!(info.hostname, None);
    }

    #[test]
    fn test_format_host_info_with_hostname() {
        let info = HostInfo::new(OsType::Linux, ArchType::X64, Some("myhost".to_string()));
        assert_eq!(format_host_info(&info), "myhost (linux/x64)");
    }

    #[test]
    fn test_format_host_info_without_hostname() {
        let info = HostInfo::without_hostname(OsType::Windows, ArchType::X64);
        assert_eq!(format_host_info(&info), "windows/x64");
    }

    #[test]
    fn test_format_rust_target_windows_x64() {
        let info = HostInfo::without_hostname(OsType::Windows, ArchType::X64);
        assert_eq!(format_rust_target(&info), "x86_64-pc-windows-msvc");
    }

    #[test]
    fn test_format_rust_target_linux_x64() {
        let info = HostInfo::without_hostname(OsType::Linux, ArchType::X64);
        assert_eq!(format_rust_target(&info), "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn test_format_rust_target_macos_arm64() {
        let info = HostInfo::without_hostname(OsType::MacOS, ArchType::Arm64);
        assert_eq!(format_rust_target(&info), "aarch64-apple-darwin");
    }

    #[test]
    fn test_format_oci_platform_linux_x64() {
        let info = HostInfo::without_hostname(OsType::Linux, ArchType::X64);
        assert_eq!(format_oci_platform(&info), "linux/amd64");
    }

    #[test]
    fn test_format_oci_platform_windows_arm64() {
        let info = HostInfo::without_hostname(OsType::Windows, ArchType::Arm64);
        assert_eq!(format_oci_platform(&info), "windows/arm64");
    }

    #[test]
    fn test_os_type_display() {
        assert_eq!(format!("{}", OsType::Linux), "linux");
        assert_eq!(format!("{}", OsType::Windows), "windows");
    }

    #[test]
    fn test_arch_type_display() {
        assert_eq!(format!("{}", ArchType::X64), "x64");
        assert_eq!(format!("{}", ArchType::Arm64), "arm64");
    }

    #[test]
    fn test_detect_functions_return_valid_values() {
        // These should not panic and should return valid enum values
        let os = detect_os();
        let arch = detect_arch();
        let host = detect_host();

        // The detected OS and arch should match what's in HostInfo
        assert_eq!(host.os, os);
        assert_eq!(host.arch, arch);
    }
}
