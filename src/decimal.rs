//! Decimal conversion utilities for 8-decimal place handling
//!
//! This module provides utilities for converting between human-readable
//! decimal formats and the internal u128 representation with 8 decimal places.

use tracing::{debug, trace};

/// Number of decimal places used internally
pub const DECIMAL_PLACES: u32 = 8;

/// Conversion factor for 8 decimal places (10^8)
pub const DECIMAL_FACTOR: u128 = 100_000_000;

/// Convert human-readable decimal string to internal u128 representation
///
/// Examples:
/// - "100.00000000" -> 100_000_000_000
/// - "100" -> 100_000_000_000
/// - "0.5" -> 50_000_000
/// - "1.23456789" -> 123_456_789
pub fn parse_decimal_amount(amount_str: &str) -> Result<u128, anyhow::Error> {
    debug!("🔍 Parsing decimal amount: {}", amount_str);

    // Remove any whitespace
    let amount_str = amount_str.trim();

    // Handle empty string
    if amount_str.is_empty() {
        return Err(anyhow::anyhow!("Empty amount string"));
    }

    // Parse as f64 first to handle decimal point
    let amount_f64 = amount_str
        .parse::<f64>()
        .map_err(|e| anyhow::anyhow!("Failed to parse amount as number: {}", e))?;

    // Check for negative amounts
    if amount_f64 < 0.0 {
        return Err(anyhow::anyhow!(
            "Negative amounts not supported: {}",
            amount_f64
        ));
    }

    // Convert to u128 with 8 decimal places
    let amount_u128 = (amount_f64 * DECIMAL_FACTOR as f64).round() as u128;

    trace!("✅ Parsed {} -> {}", amount_str, amount_u128);
    Ok(amount_u128)
}

/// Convert internal u128 representation to human-readable decimal string
///
/// Examples:
/// - 100_000_000_000 -> "100.00000000"
/// - 50_000_000 -> "0.50000000"
/// - 123_456_789 -> "1.23456789"
pub fn format_decimal_amount(amount: u128) -> String {
    let whole = amount / DECIMAL_FACTOR;
    let frac = amount % DECIMAL_FACTOR;
    format!("{}.{:0>8}", whole, frac)
}

/// Convert internal u128 representation to human-readable decimal string with trailing zeros trimmed
///
/// Examples:
/// - 100_000_000_000 -> "100"
/// - 50_000_000 -> "0.5"
/// - 123_456_789 -> "1.23456789"
pub fn format_decimal_amount_trimmed(amount: u128) -> String {
    let whole = amount / DECIMAL_FACTOR;
    let frac = amount % DECIMAL_FACTOR;

    if frac == 0 {
        whole.to_string()
    } else {
        let frac_str = format!("{:0>8}", frac);
        // Trim trailing zeros
        let trimmed = frac_str.trim_end_matches('0');
        format!("{}.{}", whole, trimmed)
    }
}

/// Format amount for LLM prompts (always with 8 decimal places for clarity)
pub fn format_amount_for_llm(amount: u128) -> String {
    format_decimal_amount(amount)
}

/// Parse amount from LLM response (handles both decimal and integer formats)
pub fn parse_amount_from_llm(amount_str: &str) -> Result<u128, anyhow::Error> {
    parse_decimal_amount(amount_str)
}

/// Convert internal u128 to display format (trimmed for user display)
pub fn format_amount_for_display(amount: u128) -> String {
    format_decimal_amount_trimmed(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decimal_amount() {
        // Test whole numbers
        assert_eq!(parse_decimal_amount("100").unwrap(), 100_000_000_000);
        assert_eq!(parse_decimal_amount("0").unwrap(), 0);

        // Test decimal numbers
        assert_eq!(
            parse_decimal_amount("100.00000000").unwrap(),
            100_000_000_000
        );
        assert_eq!(parse_decimal_amount("0.5").unwrap(), 50_000_000);
        assert_eq!(parse_decimal_amount("1.23456789").unwrap(), 123_456_789);

        // Test edge cases
        assert_eq!(parse_decimal_amount("0.00000001").unwrap(), 1);
        assert_eq!(parse_decimal_amount("0.00000000").unwrap(), 0);
    }

    #[test]
    fn test_format_decimal_amount() {
        assert_eq!(format_decimal_amount(100_000_000_000), "100.00000000");
        assert_eq!(format_decimal_amount(50_000_000), "0.50000000");
        assert_eq!(format_decimal_amount(123_456_789), "1.23456789");
        assert_eq!(format_decimal_amount(0), "0.00000000");
    }

    #[test]
    fn test_format_decimal_amount_trimmed() {
        assert_eq!(format_decimal_amount_trimmed(100_000_000_000), "100");
        assert_eq!(format_decimal_amount_trimmed(50_000_000), "0.5");
        assert_eq!(format_decimal_amount_trimmed(123_456_789), "1.23456789");
        assert_eq!(format_decimal_amount_trimmed(0), "0");
    }

    #[test]
    fn test_roundtrip_conversion() {
        let test_amounts = vec![
            "100",
            "100.00000000",
            "0.5",
            "1.23456789",
            "0.00000001",
            "0",
        ];

        for amount_str in test_amounts {
            let parsed = parse_decimal_amount(amount_str).unwrap();
            let formatted = format_decimal_amount(parsed);
            let reparsed = parse_decimal_amount(&formatted).unwrap();
            assert_eq!(parsed, reparsed, "Roundtrip failed for {}", amount_str);
        }
    }
}
