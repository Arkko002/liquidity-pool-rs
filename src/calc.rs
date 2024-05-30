use super::error::{Error, Result};
pub fn proportional(amount: u64, numerator: u64, denominator: u64) -> Result<u64> {
    if denominator == 0 {
        return Ok(amount);
    }
    u64::try_from((u128::from(amount)) * (u128::from(numerator)) / (u128::from(denominator)))
        .map_err(|_| Error::CalculationError)
}

#[inline]
pub fn value_from_shares(shares: u64, total_value: u64, total_shares: u64) -> Result<u64> {
    proportional(shares, total_value, total_shares)
}
