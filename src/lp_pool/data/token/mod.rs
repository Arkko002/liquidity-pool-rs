use std::fmt::Display;
use std::ops::{Add, AddAssign};
use std::u64;

use crate::error::{Error, Result};
use crate::lp_pool::error::Error as LpPoolError;

use super::fee::Fee;

#[derive(PartialEq, PartialOrd, Debug, Copy, Clone)]
pub struct TokenAmount(u64);
impl Add for TokenAmount {
    type Output = TokenAmount;

    fn add(self, rhs: Self) -> Self::Output {
        TokenAmount(self.0 + rhs.0)
    }
}
impl AddAssign for TokenAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Display for TokenAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<u64> for TokenAmount {
    fn into(self) -> u64 {
        self.0
    }
}

impl TokenAmount {
    pub fn from_lamports(amount: u64) -> Self {
        TokenAmount(amount)
    }

    pub fn from_staked_tokens(staked_tokens: StakedTokenAmount, price: Price) -> Self {
        TokenAmount(price.mul_by_price(staked_tokens.into()))
    }
}

pub struct LpTokenAmount(u64);
impl Add for LpTokenAmount {
    type Output = LpTokenAmount;

    fn add(self, rhs: Self) -> Self::Output {
        LpTokenAmount(self.0 + rhs.0)
    }
}
impl AddAssign for LpTokenAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Into<u64> for LpTokenAmount {
    fn into(self) -> u64 {
        self.0
    }
}

// TODO: Conversion rates and rules for all tokens
impl LpTokenAmount {
    pub fn from_tokens_with_fee(amount: TokenAmount, fee: Fee) -> Self {
        LpTokenAmount(fee.apply(amount.into()))
    }

    pub fn from_tokens_flat(amount: TokenAmount) -> Self {
        LpTokenAmount(amount.into())
    }

    pub fn from_lamports(lamports: u64) -> Self {
        LpTokenAmount(lamports)
    }
}

pub struct StakedTokenAmount(u64);

impl Add for StakedTokenAmount {
    type Output = StakedTokenAmount;

    fn add(self, rhs: Self) -> Self::Output {
        StakedTokenAmount(self.0 + rhs.0)
    }
}

impl AddAssign for StakedTokenAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Into<u64> for StakedTokenAmount {
    fn into(self) -> u64 {
        self.0
    }
}

impl StakedTokenAmount {
    // TODO: Conversion rates and rules for all tokens
    pub fn from_tokens(amount: TokenAmount, price: Price) -> Self {
        StakedTokenAmount(price.div_by_price(amount.into()))
    }

    pub fn from_lamports(lamports: u64) -> Self {
        StakedTokenAmount(lamports)
    }
}

#[derive(Debug, PartialEq)]
pub struct Price(u64);

impl TryFrom<f32> for Price {
    type Error = Error;

    fn try_from(price: f32) -> Result<Self> {
        let price_i = (price * 100.0).floor() as i64;
        let price = u32::try_from(price_i).map_err(|_| {
            Error::LpPool(LpPoolError::PriceConversionFailure {
                converted_from: price.to_string(),
            })
        })?;
        Ok(Self(price.into()))
    }
}

impl TryFrom<u64> for Price {
    type Error = Error;
    fn try_from(price_without_scale: u64) -> Result<Self> {
        let Some(price) = price_without_scale.checked_mul(100) else {
            return Err(Error::LpPool(LpPoolError::PriceConversionFailure {
                converted_from: price_without_scale.to_string(),
            }));
        };

        Ok(Self(price))
    }
}

impl std::fmt::Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{:0>2}%", self.0 / 100, self.0 % 100)
    }
}

impl Price {
    pub fn mul_by_price(&self, lamports: u64) -> u64 {
        (lamports as u128 * self.0 as u128) as u64
    }

    pub fn div_by_price(&self, lamports: u64) -> u64 {
        (lamports as u128 / self.0 as u128) as u64
    }
}
