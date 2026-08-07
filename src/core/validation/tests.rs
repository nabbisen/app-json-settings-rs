use super::{is_plain_file_name, is_safe_path_component};

const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

#[test]
fn reserved_device_names_are_rejected_upper_and_lower_case() {
    for name in RESERVED {
        assert!(!is_safe_path_component(name), "{name:?} should be rejected");
        assert!(
            !is_safe_path_component(&name.to_lowercase()),
            "{:?} should be rejected",
            name.to_lowercase()
        );
        assert!(!is_plain_file_name(name), "{name:?} should be rejected");
    }
}

#[test]
fn reserved_stems_with_extensions_are_rejected() {
    for value in ["nul.json", "CON.txt", "Com1.log", "lpt9.TXT"] {
        assert!(
            !is_safe_path_component(value),
            "{value:?} should be rejected"
        );
        assert!(!is_plain_file_name(value), "{value:?} should be rejected");
    }
}

#[test]
fn names_merely_starting_with_a_reserved_string_are_accepted() {
    for value in ["console", "nullable", "communications", "printer"] {
        assert!(is_safe_path_component(value), "{value:?} should be valid");
        assert!(is_plain_file_name(value), "{value:?} should be valid");
    }
}

#[test]
fn existing_valid_names_are_still_accepted() {
    for value in ["settings.json", "my-app"] {
        assert!(is_safe_path_component(value), "{value:?} should be valid");
        assert!(is_plain_file_name(value), "{value:?} should be valid");
    }
}
