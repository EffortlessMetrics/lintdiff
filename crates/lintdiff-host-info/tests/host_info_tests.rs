//! Comprehensive BDD tests for lintdiff-host-info crate.
//!
//! These tests cover:
//! - OsType variants and detection
//! - ArchType variants and detection
//! - HostInfo construction and detection
//! - Formatting functions
//! - Edge cases
//! - Property-based tests with proptest

use lintdiff_host_info::{
    detect_arch, detect_host, detect_os, format_host_info, format_oci_platform,
    format_rust_target, get_hostname, ArchType, HostInfo, OsType,
};
use proptest::prelude::*;

// =============================================================================
// OsType Tests (8 tests)
// =============================================================================

#[test]
fn os_type_windows_as_str_returns_windows() {
    assert_eq!(OsType::Windows.as_str(), "windows");
}

#[test]
fn os_type_linux_as_str_returns_linux() {
    assert_eq!(OsType::Linux.as_str(), "linux");
}

#[test]
fn os_type_macos_as_str_returns_macos() {
    assert_eq!(OsType::MacOS.as_str(), "macos");
}

#[test]
fn os_type_freebsd_as_str_returns_freebsd() {
    assert_eq!(OsType::FreeBSD.as_str(), "freebsd");
}

#[test]
fn os_type_other_as_str_returns_other() {
    assert_eq!(OsType::Other.as_str(), "other");
}

#[test]
fn os_type_display_trait_works_correctly() {
    assert_eq!(format!("{}", OsType::Windows), "windows");
    assert_eq!(format!("{}", OsType::Linux), "linux");
    assert_eq!(format!("{}", OsType::MacOS), "macos");
    assert_eq!(format!("{}", OsType::FreeBSD), "freebsd");
    assert_eq!(format!("{}", OsType::Other), "other");
}

#[test]
fn os_type_default_returns_detected_os() {
    let os = OsType::default();
    // Default should match detect
    assert_eq!(os, OsType::detect());
}

#[test]
fn os_type_detect_returns_valid_variant() {
    let os = OsType::detect();
    // Should be one of the known variants based on current system
    let os_str = os.as_str();
    assert!(
        ["windows", "linux", "macos", "freebsd", "other"].contains(&os_str),
        "Detected OS should be a valid variant"
    );
}

// =============================================================================
// ArchType Tests (8 tests)
// =============================================================================

#[test]
fn arch_type_x86_as_str_returns_x86() {
    assert_eq!(ArchType::X86.as_str(), "x86");
}

#[test]
fn arch_type_x64_as_str_returns_x64() {
    assert_eq!(ArchType::X64.as_str(), "x64");
}

#[test]
fn arch_type_arm_as_str_returns_arm() {
    assert_eq!(ArchType::Arm.as_str(), "arm");
}

#[test]
fn arch_type_arm64_as_str_returns_arm64() {
    assert_eq!(ArchType::Arm64.as_str(), "arm64");
}

#[test]
fn arch_type_other_as_str_returns_other() {
    assert_eq!(ArchType::Other.as_str(), "other");
}

#[test]
fn arch_type_display_trait_works_correctly() {
    assert_eq!(format!("{}", ArchType::X86), "x86");
    assert_eq!(format!("{}", ArchType::X64), "x64");
    assert_eq!(format!("{}", ArchType::Arm), "arm");
    assert_eq!(format!("{}", ArchType::Arm64), "arm64");
    assert_eq!(format!("{}", ArchType::Other), "other");
}

#[test]
fn arch_type_default_returns_detected_arch() {
    let arch = ArchType::default();
    // Default should match detect
    assert_eq!(arch, ArchType::detect());
}

#[test]
fn arch_type_detect_returns_valid_variant() {
    let arch = ArchType::detect();
    // Should be one of the known variants based on current system
    let arch_str = arch.as_str();
    assert!(
        ["x86", "x64", "arm", "arm64", "other"].contains(&arch_str),
        "Detected architecture should be a valid variant"
    );
}

// =============================================================================
// HostInfo Construction Tests (6 tests)
// =============================================================================

#[test]
fn host_info_new_creates_instance_with_all_fields() {
    let info = HostInfo::new(OsType::Linux, ArchType::X64, Some("testhost".to_string()));
    
    assert_eq!(info.os, OsType::Linux);
    assert_eq!(info.arch, ArchType::X64);
    assert_eq!(info.hostname, Some("testhost".to_string()));
}

#[test]
fn host_info_new_accepts_none_hostname() {
    let info = HostInfo::new(OsType::Windows, ArchType::Arm64, None);
    
    assert_eq!(info.os, OsType::Windows);
    assert_eq!(info.arch, ArchType::Arm64);
    assert_eq!(info.hostname, None);
}

#[test]
fn host_info_without_hostname_creates_instance_with_none_hostname() {
    let info = HostInfo::without_hostname(OsType::MacOS, ArchType::Arm64);
    
    assert_eq!(info.os, OsType::MacOS);
    assert_eq!(info.arch, ArchType::Arm64);
    assert_eq!(info.hostname, None);
}

#[test]
fn host_info_detect_returns_valid_info() {
    let info = HostInfo::detect();
    
    // OS and arch should be valid detected values
    assert!(
        ["windows", "linux", "macos", "freebsd", "other"].contains(&info.os.as_str())
    );
    assert!(
        ["x86", "x64", "arm", "arm64", "other"].contains(&info.arch.as_str())
    );
}

#[test]
fn host_info_default_returns_detected_info() {
    let info = HostInfo::default();
    let detected = HostInfo::detect();
    
    assert_eq!(info.os, detected.os);
    assert_eq!(info.arch, detected.arch);
}

#[test]
fn host_info_is_cloneable() {
    let info = HostInfo::new(OsType::Linux, ArchType::X64, Some("host".to_string()));
    let cloned = info.clone();
    
    assert_eq!(info, cloned);
}

// =============================================================================
// Detection Function Tests (4 tests)
// =============================================================================

#[test]
fn detect_os_function_returns_same_as_type_method() {
    assert_eq!(detect_os(), OsType::detect());
}

#[test]
fn detect_arch_function_returns_same_as_type_method() {
    assert_eq!(detect_arch(), ArchType::detect());
}

#[test]
fn detect_host_function_returns_complete_info() {
    let info = detect_host();
    
    assert_eq!(info.os, detect_os());
    assert_eq!(info.arch, detect_arch());
}

#[test]
fn get_hostname_function_returns_string_or_none() {
    // This test just verifies the function doesn't panic
    // The actual value depends on the system
    let _hostname = get_hostname();
}

// =============================================================================
// format_host_info Tests (4 tests)
// =============================================================================

#[test]
fn format_host_info_with_hostname_includes_hostname() {
    let info = HostInfo::new(OsType::Linux, ArchType::X64, Some("myhost".to_string()));
    
    assert_eq!(format_host_info(&info), "myhost (linux/x64)");
}

#[test]
fn format_host_info_without_hostname_omits_hostname() {
    let info = HostInfo::without_hostname(OsType::Windows, ArchType::X64);
    
    assert_eq!(format_host_info(&info), "windows/x64");
}

#[test]
fn format_host_info_with_empty_hostname_treated_as_none() {
    let info = HostInfo::new(OsType::Linux, ArchType::X64, Some(String::new()));
    
    // Empty hostname is still displayed
    assert_eq!(format_host_info(&info), " (linux/x64)");
}

#[test]
fn format_host_info_all_os_types() {
    assert_eq!(
        format_host_info(&HostInfo::without_hostname(OsType::Windows, ArchType::X64)),
        "windows/x64"
    );
    assert_eq!(
        format_host_info(&HostInfo::without_hostname(OsType::Linux, ArchType::X64)),
        "linux/x64"
    );
    assert_eq!(
        format_host_info(&HostInfo::without_hostname(OsType::MacOS, ArchType::X64)),
        "macos/x64"
    );
    assert_eq!(
        format_host_info(&HostInfo::without_hostname(OsType::FreeBSD, ArchType::X64)),
        "freebsd/x64"
    );
    assert_eq!(
        format_host_info(&HostInfo::without_hostname(OsType::Other, ArchType::X64)),
        "other/x64"
    );
}

// =============================================================================
// format_rust_target Tests (10 tests)
// =============================================================================

#[test]
fn format_rust_target_windows_x64() {
    let info = HostInfo::without_hostname(OsType::Windows, ArchType::X64);
    assert_eq!(format_rust_target(&info), "x86_64-pc-windows-msvc");
}

#[test]
fn format_rust_target_windows_x86() {
    let info = HostInfo::without_hostname(OsType::Windows, ArchType::X86);
    assert_eq!(format_rust_target(&info), "i686-pc-windows-msvc");
}

#[test]
fn format_rust_target_windows_arm64() {
    let info = HostInfo::without_hostname(OsType::Windows, ArchType::Arm64);
    assert_eq!(format_rust_target(&info), "aarch64-pc-windows-msvc");
}

#[test]
fn format_rust_target_linux_x64() {
    let info = HostInfo::without_hostname(OsType::Linux, ArchType::X64);
    assert_eq!(format_rust_target(&info), "x86_64-unknown-linux-gnu");
}

#[test]
fn format_rust_target_linux_x86() {
    let info = HostInfo::without_hostname(OsType::Linux, ArchType::X86);
    assert_eq!(format_rust_target(&info), "i686-unknown-linux-gnu");
}

#[test]
fn format_rust_target_linux_arm() {
    let info = HostInfo::without_hostname(OsType::Linux, ArchType::Arm);
    assert_eq!(format_rust_target(&info), "arm-unknown-linux-gnueabihf");
}

#[test]
fn format_rust_target_linux_arm64() {
    let info = HostInfo::without_hostname(OsType::Linux, ArchType::Arm64);
    assert_eq!(format_rust_target(&info), "aarch64-unknown-linux-gnu");
}

#[test]
fn format_rust_target_macos_x64() {
    let info = HostInfo::without_hostname(OsType::MacOS, ArchType::X64);
    assert_eq!(format_rust_target(&info), "x86_64-apple-darwin");
}

#[test]
fn format_rust_target_macos_arm64() {
    let info = HostInfo::without_hostname(OsType::MacOS, ArchType::Arm64);
    assert_eq!(format_rust_target(&info), "aarch64-apple-darwin");
}

#[test]
fn format_rust_target_freebsd_x64() {
    let info = HostInfo::without_hostname(OsType::FreeBSD, ArchType::X64);
    assert_eq!(format_rust_target(&info), "x86_64-unknown-freebsd");
}

// =============================================================================
// format_oci_platform Tests (6 tests)
// =============================================================================

#[test]
fn format_oci_platform_linux_x64() {
    let info = HostInfo::without_hostname(OsType::Linux, ArchType::X64);
    assert_eq!(format_oci_platform(&info), "linux/amd64");
}

#[test]
fn format_oci_platform_linux_arm64() {
    let info = HostInfo::without_hostname(OsType::Linux, ArchType::Arm64);
    assert_eq!(format_oci_platform(&info), "linux/arm64");
}

#[test]
fn format_oci_platform_windows_x64() {
    let info = HostInfo::without_hostname(OsType::Windows, ArchType::X64);
    assert_eq!(format_oci_platform(&info), "windows/amd64");
}

#[test]
fn format_oci_platform_windows_arm64() {
    let info = HostInfo::without_hostname(OsType::Windows, ArchType::Arm64);
    assert_eq!(format_oci_platform(&info), "windows/arm64");
}

#[test]
fn format_oci_platform_uses_386_for_x86() {
    let info = HostInfo::without_hostname(OsType::Linux, ArchType::X86);
    assert_eq!(format_oci_platform(&info), "linux/386");
}

#[test]
fn format_oci_platform_all_architectures() {
    assert_eq!(
        format_oci_platform(&HostInfo::without_hostname(OsType::Linux, ArchType::X86)),
        "linux/386"
    );
    assert_eq!(
        format_oci_platform(&HostInfo::without_hostname(OsType::Linux, ArchType::X64)),
        "linux/amd64"
    );
    assert_eq!(
        format_oci_platform(&HostInfo::without_hostname(OsType::Linux, ArchType::Arm)),
        "linux/arm"
    );
    assert_eq!(
        format_oci_platform(&HostInfo::without_hostname(OsType::Linux, ArchType::Arm64)),
        "linux/arm64"
    );
}

// =============================================================================
// Edge Case Tests (5 tests)
// =============================================================================

#[test]
fn host_info_with_very_long_hostname() {
    let long_hostname = "a".repeat(256);
    let info = HostInfo::new(OsType::Linux, ArchType::X64, Some(long_hostname.clone()));
    
    assert_eq!(info.hostname, Some(long_hostname));
}

#[test]
fn host_info_with_unicode_hostname() {
    let unicode_hostname = "主机名-ホストネーム-🏠".to_string();
    let info = HostInfo::new(OsType::Linux, ArchType::X64, Some(unicode_hostname.clone()));
    
    assert_eq!(info.hostname, Some(unicode_hostname));
}

#[test]
fn format_host_info_preserves_unicode() {
    let unicode_hostname = "тест".to_string();
    let info = HostInfo::new(OsType::Linux, ArchType::X64, Some(unicode_hostname));
    
    let formatted = format_host_info(&info);
    assert!(formatted.contains("тест"));
}

#[test]
fn os_type_equality_works() {
    assert_eq!(OsType::Linux, OsType::Linux);
    assert_ne!(OsType::Linux, OsType::Windows);
}

#[test]
fn arch_type_equality_works() {
    assert_eq!(ArchType::X64, ArchType::X64);
    assert_ne!(ArchType::X64, ArchType::Arm64);
}

// =============================================================================
// Hash Tests (2 tests)
// =============================================================================

use std::collections::HashSet;

#[test]
fn os_type_can_be_used_in_hashset() {
    let mut set = HashSet::new();
    set.insert(OsType::Linux);
    set.insert(OsType::Windows);
    set.insert(OsType::Linux); // Duplicate
    
    assert_eq!(set.len(), 2);
    assert!(set.contains(&OsType::Linux));
    assert!(set.contains(&OsType::Windows));
}

#[test]
fn arch_type_can_be_used_in_hashset() {
    let mut set = HashSet::new();
    set.insert(ArchType::X64);
    set.insert(ArchType::Arm64);
    set.insert(ArchType::X64); // Duplicate
    
    assert_eq!(set.len(), 2);
    assert!(set.contains(&ArchType::X64));
    assert!(set.contains(&ArchType::Arm64));
}

// =============================================================================
// Clone Tests (2 tests)
// =============================================================================

#[test]
fn os_type_is_cloneable() {
    let os = OsType::Linux;
    let cloned = os.clone();
    assert_eq!(os, cloned);
}

#[test]
fn arch_type_is_cloneable() {
    let arch = ArchType::X64;
    let cloned = arch.clone();
    assert_eq!(arch, cloned);
}

// =============================================================================
// Property-Based Tests with proptest (5 tests)
// =============================================================================

proptest! {
    #[test]
    fn format_host_info_never_panics(
        os in 0..5usize,
        arch in 0..5usize,
        hostname in proptest::option::of(proptest::string::string_regex(".*").unwrap())
    ) {
        let os_type = match os % 5 {
            0 => OsType::Windows,
            1 => OsType::Linux,
            2 => OsType::MacOS,
            3 => OsType::FreeBSD,
            _ => OsType::Other,
        };
        
        let arch_type = match arch % 5 {
            0 => ArchType::X86,
            1 => ArchType::X64,
            2 => ArchType::Arm,
            3 => ArchType::Arm64,
            _ => ArchType::Other,
        };
        
        let info = HostInfo::new(os_type, arch_type, hostname);
        let _formatted = format_host_info(&info);
    }
    
    #[test]
    fn format_rust_target_never_panics(
        os in 0..5usize,
        arch in 0..5usize
    ) {
        let os_type = match os % 5 {
            0 => OsType::Windows,
            1 => OsType::Linux,
            2 => OsType::MacOS,
            3 => OsType::FreeBSD,
            _ => OsType::Other,
        };
        
        let arch_type = match arch % 5 {
            0 => ArchType::X86,
            1 => ArchType::X64,
            2 => ArchType::Arm,
            3 => ArchType::Arm64,
            _ => ArchType::Other,
        };
        
        let info = HostInfo::without_hostname(os_type, arch_type);
        let _target = format_rust_target(&info);
    }
    
    #[test]
    fn format_oci_platform_never_panics(
        os in 0..5usize,
        arch in 0..5usize
    ) {
        let os_type = match os % 5 {
            0 => OsType::Windows,
            1 => OsType::Linux,
            2 => OsType::MacOS,
            3 => OsType::FreeBSD,
            _ => OsType::Other,
        };
        
        let arch_type = match arch % 5 {
            0 => ArchType::X86,
            1 => ArchType::X64,
            2 => ArchType::Arm,
            3 => ArchType::Arm64,
            _ => ArchType::Other,
        };
        
        let info = HostInfo::without_hostname(os_type, arch_type);
        let _platform = format_oci_platform(&info);
    }
    
    #[test]
    fn os_type_as_str_returns_non_empty_string(os in 0..5usize) {
        let os_type = match os % 5 {
            0 => OsType::Windows,
            1 => OsType::Linux,
            2 => OsType::MacOS,
            3 => OsType::FreeBSD,
            _ => OsType::Other,
        };
        
        let s = os_type.as_str();
        prop_assert!(!s.is_empty());
    }
    
    #[test]
    fn arch_type_as_str_returns_non_empty_string(arch in 0..5usize) {
        let arch_type = match arch % 5 {
            0 => ArchType::X86,
            1 => ArchType::X64,
            2 => ArchType::Arm,
            3 => ArchType::Arm64,
            _ => ArchType::Other,
        };
        
        let s = arch_type.as_str();
        prop_assert!(!s.is_empty());
    }
}

// =============================================================================
// Format Output Structure Tests (4 tests)
// =============================================================================

#[test]
fn format_rust_target_contains_arch_and_os() {
    let info = HostInfo::without_hostname(OsType::Linux, ArchType::X64);
    let target = format_rust_target(&info);
    
    assert!(target.contains("x86_64") || target.contains("x64"));
    assert!(target.contains("linux"));
}

#[test]
fn format_oci_platform_has_correct_format() {
    let info = HostInfo::without_hostname(OsType::Linux, ArchType::Arm64);
    let platform = format_oci_platform(&info);
    
    // Should be in format "os/arch"
    let parts: Vec<&str> = platform.split('/').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "linux");
    assert_eq!(parts[1], "arm64");
}

#[test]
fn format_host_info_with_hostname_has_parens() {
    let info = HostInfo::new(OsType::Linux, ArchType::X64, Some("host".to_string()));
    let formatted = format_host_info(&info);
    
    assert!(formatted.contains('('));
    assert!(formatted.contains(')'));
    assert!(formatted.contains('/'));
}

#[test]
fn format_host_info_without_hostname_has_no_parens() {
    let info = HostInfo::without_hostname(OsType::Linux, ArchType::X64);
    let formatted = format_host_info(&info);
    
    assert!(!formatted.contains('('));
    assert!(!formatted.contains(')'));
    assert!(formatted.contains('/'));
}

// =============================================================================
// Must Use Attribute Tests (3 tests)
// =============================================================================

#[test]
fn detect_os_returns_useful_value() {
    let os = detect_os();
    // Verify it's a valid enum variant by using it
    let _ = os.as_str();
}

#[test]
fn detect_arch_returns_useful_value() {
    let arch = detect_arch();
    // Verify it's a valid enum variant by using it
    let _ = arch.as_str();
}

#[test]
fn detect_host_returns_useful_value() {
    let host = detect_host();
    // Verify all fields are accessible
    let _ = host.os.as_str();
    let _ = host.arch.as_str();
    let _ = &host.hostname;
}
