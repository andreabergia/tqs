use std::path::Path;

use crate::app::app_error::AppError;

pub const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
pub const MIN_GENERATED_ID_WIDTH: u8 = 3;
pub const MAX_GENERATED_ID_WIDTH: u8 = 25;
pub const SEQUENCE_STEP: u128 = 0x9e3779b97f4a7c15;

pub fn validate_user_id(id: &str) -> Result<(), AppError> {
    let trimmed = id.trim();

    if trimmed.is_empty() {
        return Err(AppError::usage("task ID cannot be empty"));
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(AppError::usage("task ID cannot be an absolute path"));
    }

    if trimmed.starts_with('.') {
        return Err(AppError::usage("task ID cannot start with '.'"));
    }

    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(AppError::usage(
            "task ID cannot contain path separators (/ or \\)",
        ));
    }

    Ok(())
}

pub fn encode_generated_id(mut value: u128, width: u8) -> Result<String, AppError> {
    validate_generated_width(width)?;

    let modulus = id_space_size(width)?;
    if value >= modulus {
        return Err(AppError::message(format!(
            "generated ID value {value} is out of range for width {width}"
        )));
    }

    let mut output = vec!['0'; usize::from(width)];
    for index in (0..usize::from(width)).rev() {
        let digit = (value % 32) as usize;
        output[index] = char::from(CROCKFORD_ALPHABET[digit]);
        value /= 32;
    }

    Ok(output.into_iter().collect())
}

pub fn id_space_size(width: u8) -> Result<u128, AppError> {
    validate_generated_width(width)?;
    Ok(1u128 << (u32::from(width) * 5))
}

pub fn next_sequence_value(current: u128, width: u8) -> Result<u128, AppError> {
    let modulus = id_space_size(width)?;
    Ok((current + (SEQUENCE_STEP % modulus)) % modulus)
}

fn validate_generated_width(width: u8) -> Result<(), AppError> {
    if !(MIN_GENERATED_ID_WIDTH..=MAX_GENERATED_ID_WIDTH).contains(&width) {
        return Err(AppError::message(format!(
            "generated ID width {width} is unsupported"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        CROCKFORD_ALPHABET, MIN_GENERATED_ID_WIDTH, encode_generated_id, id_space_size,
        next_sequence_value, validate_user_id,
    };

    #[test]
    fn generated_ids_use_lowercase_crockford_alphabet() {
        let id = encode_generated_id(32 * 32 + 10 * 32 + 31, MIN_GENERATED_ID_WIDTH)
            .expect("generated ID should encode");
        assert_eq!(id, "1az");
        assert!(id.bytes().all(|byte| CROCKFORD_ALPHABET.contains(&byte)));
    }

    #[test]
    fn generated_ids_are_zero_padded_to_width() {
        assert_eq!(
            encode_generated_id(1, MIN_GENERATED_ID_WIDTH).expect("generated ID should encode"),
            "001"
        );
    }

    #[test]
    fn additive_sequence_covers_the_full_width_without_repeating() {
        let width = MIN_GENERATED_ID_WIDTH;
        let modulus = id_space_size(width).expect("space size should compute");
        let mut value = 0;
        let mut seen = HashSet::new();

        for _ in 0..modulus {
            assert!(seen.insert(value), "sequence repeated before wrap");
            value = next_sequence_value(value, width).expect("sequence should advance");
        }

        assert_eq!(value, 0, "sequence should wrap to zero after full cycle");
        assert_eq!(
            seen.len() as u128,
            modulus,
            "sequence should visit all slots"
        );
    }

    mod validation_tests {
        use super::validate_user_id;

        #[test]
        fn valid_id_with_hyphens_succeeds() {
            assert!(validate_user_id("task-123").is_ok());
        }

        #[test]
        fn valid_id_with_underscores_succeeds() {
            assert!(validate_user_id("task_123").is_ok());
        }

        #[test]
        fn valid_id_with_spaces_succeeds() {
            assert!(validate_user_id("my task").is_ok());
        }

        #[test]
        fn valid_id_with_unicode_succeeds() {
            assert!(validate_user_id("tâche-123").is_ok());
        }

        #[test]
        fn valid_id_with_emoji_succeeds() {
            assert!(validate_user_id("task-✅").is_ok());
        }

        #[test]
        fn valid_id_with_mixed_chars_succeeds() {
            assert!(validate_user_id("task-123_测试").is_ok());
        }

        #[test]
        fn valid_id_with_dots_succeeds() {
            assert!(validate_user_id("task.1.2").is_ok());
        }

        #[test]
        fn empty_id_fails() {
            let result = validate_user_id("");
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("cannot be empty"));
        }

        #[test]
        fn whitespace_only_id_fails() {
            let result = validate_user_id("   ");
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("cannot be empty"));
        }

        #[test]
        fn dot_id_fails() {
            let result = validate_user_id(".");
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("start with '.'"));
        }

        #[test]
        fn double_dot_id_fails() {
            let result = validate_user_id("..");
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("start with '.'"));
        }

        #[test]
        fn hidden_file_style_id_fails() {
            let result = validate_user_id(".task-123");
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("start with '.'"));
        }

        #[test]
        fn absolute_path_fails() {
            let result = validate_user_id("/etc/passwd");
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("absolute path"));
        }

        #[test]
        fn path_separators_fail() {
            let result = validate_user_id("foo/bar");
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("path separators"));
        }
    }
}
