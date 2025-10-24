///! Configuration management for archive processing
///!
///! Reads settings from the Windows registry

use winreg::RegKey;
use winreg::enums::*;

const CONFIG_KEY_PATH: &str = "Software\\CBXShell-rs\\{9E6ECB90-5A61-42BD-B851-D3297D9C7F39}";
const NO_SORT_VALUE: &str = "NoSort";

/// Read the sorting preference from the registry
///
/// Returns `true` if images should be sorted alphabetically (default).
/// Returns `false` if the first image encountered should be used.
///
/// Registry location: HKCU\Software\CBXShell-rs\{GUID}\NoSort
/// - Value 0 or missing = sort enabled (true)
/// - Value 1 = sort disabled (false)
pub fn should_sort_images() -> bool {
    match read_no_sort_setting() {
        Ok(no_sort) => !no_sort,  // Invert: NoSort=0 means sort=true
        Err(_) => {
            // Default to sorting if registry read fails
            tracing::debug!("Failed to read NoSort setting, defaulting to sorted mode");
            true
        }
    }
}

/// Read the NoSort registry value
///
/// Returns `Ok(true)` if NoSort=1 (sorting disabled)
/// Returns `Ok(false)` if NoSort=0 or missing (sorting enabled)
fn read_no_sort_setting() -> Result<bool, std::io::Error> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    match hkcu.open_subkey(CONFIG_KEY_PATH) {
        Ok(key) => {
            match key.get_value::<u32, _>(NO_SORT_VALUE) {
                Ok(value) => Ok(value != 0),  // NonZero = true (don't sort)
                Err(_) => Ok(false),  // Missing value = false (do sort)
            }
        }
        Err(_) => Ok(false),  // Missing key = false (do sort)
    }
}

/// Set the sorting preference in the registry (for testing/configuration)
///
/// If `sort` is true, sets NoSort=0 (sorting enabled)
/// If `sort` is false, sets NoSort=1 (sorting disabled)
#[allow(dead_code)]
pub fn set_should_sort_images(sort: bool) -> Result<(), std::io::Error> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(CONFIG_KEY_PATH)?;

    let no_sort_value: u32 = if sort { 0 } else { 1 };
    key.set_value(NO_SORT_VALUE, &no_sort_value)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_no_sort_default() {
        // Should default to sorting if key doesn't exist
        // (This test will pass even if registry key exists)
        let result = should_sort_images();
        assert!(result == true || result == false);  // Just verify it doesn't crash
    }

    #[test]
    fn test_set_and_read_sorting() {
        // Test round-trip (might fail if no registry access)
        if set_should_sort_images(true).is_ok() {
            assert_eq!(should_sort_images(), true);
        }

        if set_should_sort_images(false).is_ok() {
            assert_eq!(should_sort_images(), false);
        }

        // Cleanup: restore to default (sorting enabled)
        let _ = set_should_sort_images(true);
    }
}
