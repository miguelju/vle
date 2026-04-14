//! Parser for `"<value> <unit>"` strings.
//!
//! Examples:
//! - `"25 degC"` → `(25.0, "degC")`
//! - `"3.5 barg"` → `(3.5, "barg")`
//! - `"-1.2e-3 kJ/kmol"` → `(-1.2e-3, "kJ/kmol")`

use crate::registry::RegistryError;

/// Split a string of the form `"<numeric value> <unit symbol>"` into its
/// `(value, unit)` parts. Whitespace between the two is required.
pub fn split_value_unit(s: &str) -> Result<(f64, &str), RegistryError> {
    let trimmed = s.trim();
    // Find first ASCII whitespace — value cannot contain spaces
    let split_at = trimmed.find(char::is_whitespace).ok_or_else(|| {
        RegistryError::ParseError(format!("expected '<value> <unit>', got '{s}'"))
    })?;
    let (val_str, rest) = trimmed.split_at(split_at);
    let unit = rest.trim_start();
    if unit.is_empty() {
        return Err(RegistryError::ParseError(format!(
            "missing unit after value in '{s}'"
        )));
    }
    let value: f64 = val_str
        .parse()
        .map_err(|_| RegistryError::ParseError(format!("invalid number '{val_str}'")))?;
    Ok((value, unit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple() {
        assert_eq!(split_value_unit("25 degC").unwrap(), (25.0, "degC"));
        assert_eq!(split_value_unit("3.5 barg").unwrap(), (3.5, "barg"));
        assert_eq!(
            split_value_unit("-1.2e-3 kJ/kmol").unwrap(),
            (-1.2e-3, "kJ/kmol")
        );
    }

    #[test]
    fn rejects_missing_unit() {
        assert!(split_value_unit("25").is_err());
        assert!(split_value_unit("25 ").is_err());
    }

    #[test]
    fn rejects_bad_number() {
        assert!(split_value_unit("abc kPa").is_err());
    }
}
