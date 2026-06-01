//! Capability detection for data collection backends
//!
//! Determines which backends are available at runtime so the collector
//! can make informed fallback decisions and the UI can label data sources
//! accurately.

use serde::Serialize;

/// Runtime capabilities of the current system
#[derive(Debug, Clone, Serialize, Default)]
pub struct CollectorCapabilities {
    pub powermetrics_available: bool,
    pub powermetrics_privileged: bool,
    pub smc_read_available: bool,
    pub smc_write_available: bool,
    pub ioreport_available: bool,
    pub iohid_available: bool,
    pub foundation_available: bool,
    pub mach_available: bool,
}

impl CollectorCapabilities {
    /// Detect all capabilities at startup
    pub fn detect() -> Self {
        Self {
            powermetrics_available: Self::check_powermetrics(),
            powermetrics_privileged: Self::check_root(),
            smc_read_available: Self::check_smc_read(),
            smc_write_available: false,
            ioreport_available: Self::check_ioreport(),
            iohid_available: Self::check_iohid(),
            foundation_available: Self::check_foundation(),
            mach_available: true, // Mach is always available on macOS
        }
    }

    fn check_smc_read() -> bool {
        #[cfg(target_os = "macos")]
        {
            crate::collectors::backends::SmcHandle::open().is_ok()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn check_powermetrics() -> bool {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("which")
                .arg("powermetrics")
                .output()
                .ok()
                .is_some_and(|o| o.status.success())
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn check_root() -> bool {
        #[cfg(target_os = "macos")]
        {
            unsafe { libc::getuid() == 0 }
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn check_ioreport() -> bool {
        #[cfg(target_os = "macos")]
        {
            crate::collectors::backends::is_ioreport_available()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn check_iohid() -> bool {
        #[cfg(target_os = "macos")]
        {
            crate::collectors::backends::is_iohid_available()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn check_foundation() -> bool {
        #[cfg(target_os = "macos")]
        {
            // Foundation is always present on macOS
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_default() {
        let caps = CollectorCapabilities::default();
        assert!(!caps.powermetrics_available);
        assert!(!caps.powermetrics_privileged);
        assert!(!caps.smc_read_available);
        assert!(!caps.smc_write_available);
        assert!(!caps.ioreport_available);
        assert!(!caps.iohid_available);
        assert!(!caps.foundation_available);
        assert!(!caps.mach_available);
    }

    #[test]
    fn test_capabilities_detect_has_expected_fields() {
        let caps = CollectorCapabilities::detect();
        // These should always be detectable (even if false)
        assert!(
            !caps.smc_write_available,
            "smc write should always be false (read-only)"
        );
        // powermetrics availability depends on the system
        let _ = caps.powermetrics_available;
        let _ = caps.mach_available;
    }

    #[test]
    fn test_capabilities_serialization() {
        let caps = CollectorCapabilities::default();
        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains("powermetrics_available"));
        assert!(json.contains("mach_available"));
    }
}
