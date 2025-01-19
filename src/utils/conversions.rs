use apibara_core::starknet::v1alpha2::FieldElement;
use bigdecimal::num_bigint::BigInt;
use bigdecimal::BigDecimal;
use starknet::core::types::{Felt, U256};

/// Converts an hexadecimal string with decimals to BigDecimal.
pub fn hex_str_to_big_decimal(hex_price: &str, decimals: i64) -> BigDecimal {
    let cleaned_hex = hex_price.trim_start_matches("0x");
    let price_bigint = BigInt::parse_bytes(cleaned_hex.as_bytes(), 16).unwrap();
    BigDecimal::new(price_bigint, decimals)
}

/// Converts a Felt element from starknet-rs to a FieldElement from Apibara-core.
pub fn felt_as_apibara_field(value: &Felt) -> FieldElement {
    FieldElement::from_bytes(&value.to_bytes_be())
}

/// Converts an Apibara core FieldElement into a Felt from starknet-rs.
pub fn apibara_field_as_felt(value: &FieldElement) -> Felt {
    Felt::from_bytes_be(&value.to_bytes())
}

/// Converts a BigDecimal to a U256.
pub fn big_decimal_to_u256(value: BigDecimal) -> U256 {
    U256::from(big_decimal_to_felt(value))
}

pub fn big_decimal_to_felt(value: BigDecimal) -> Felt {
    let (amount, _): (BigInt, _) = value.as_bigint_and_exponent();
    Felt::from(amount.clone())
}

// Helper function to convert FieldElement to hex string
pub fn field_to_hex_string(field: &apibara_core::starknet::v1alpha2::FieldElement) -> String {
    format!("0x{:016x}{:016x}{:016x}{:016x}", 
        field.hi_hi,
        field.hi_lo,
        field.lo_hi,
        field.lo_lo
    )
}

// Helper function to convert hex string to UTF-8 string
fn hex_to_string(hex: &str) -> String {
    // Remove 0x prefix if present
    let hex_clean = hex.trim_start_matches("0x");
    
    // Convert hex to bytes
    let bytes: Vec<u8> = (0..hex_clean.len())
        .step_by(2)
        .filter_map(|i| {
            if i + 2 <= hex_clean.len() {
                u8::from_str_radix(&hex_clean[i..i + 2], 16).ok()
            } else {
                None
            }
        })
        .collect();
    
    // Convert bytes to string, removing null bytes
    String::from_utf8(bytes)
        .map(|s| s.trim_matches(char::from(0)).to_string())
        .unwrap_or_else(|_| hex.to_string())
}

pub fn field_to_string(field: &apibara_core::starknet::v1alpha2::FieldElement) -> String {
    let hex = field_to_hex_string(field);
    
    // Try to convert to string
    let result = hex_to_string(&hex);
    
    // If result is empty or only contains null bytes, return original hex
    if result.trim().is_empty() {
        hex
    } else {
        result
    }
}