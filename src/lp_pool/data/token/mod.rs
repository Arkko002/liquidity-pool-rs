use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, Sub, SubAssign};
use std::u64;

use super::fee::Fee;
use super::price::Price;
use crate::error::Result;

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, PartialOrd, Debug, Copy, Clone)]
pub struct TokenAmount(u64);
impl Add for TokenAmount {
    type Output = TokenAmount;

    fn add(self, rhs: Self) -> Self::Output {
        let Some(resultult) = self.0.checked_add(rhs.0) else {
            panic!("Overflow in TokenAmount::add");
        };
        TokenAmount(resultult)
    }
}
impl AddAssign for TokenAmount {
    fn add_assign(&mut self, rhs: Self) {
        let Some(result) = self.0.checked_add(rhs.0) else {
            panic!("Overflow in TokenAmount::add_assign");
        };
        self.0 = result;
    }
}

impl Sub for TokenAmount {
    type Output = TokenAmount;

    fn sub(self, rhs: Self) -> Self::Output {
        let Some(result) = self.0.checked_sub(rhs.0) else {
            panic!("Overflow in TokenAmount::sub");
        };
        TokenAmount(result)
    }
}

impl SubAssign for TokenAmount {
    fn sub_assign(&mut self, rhs: Self) {
        let Some(result) = self.0.checked_sub(rhs.0) else {
            panic!("Overflow in TokenAmount::sub_assign");
        };
        self.0 = result;
    }
}

impl Div for TokenAmount {
    type Output = TokenAmount;

    fn div(self, rhs: Self) -> Self::Output {
        let Some(result) = self.0.checked_div(rhs.0) else {
            panic!("Overflow in TokenAmount::div");
        };
        TokenAmount(result)
    }
}

impl Display for TokenAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TokenAmount> for u64 {
    fn from(val: TokenAmount) -> Self {
        val.0
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LpTokenAmount(u64);
impl Add for LpTokenAmount {
    type Output = LpTokenAmount;

    fn add(self, rhs: Self) -> Self::Output {
        let Some(result) = self.0.checked_add(rhs.0) else {
            panic!("Overflow in LpTokenAmount::add");
        };
        LpTokenAmount(result)
    }
}
impl AddAssign for LpTokenAmount {
    fn add_assign(&mut self, rhs: Self) {
        let Some(result) = self.0.checked_add(rhs.0) else {
            panic!("Overflow in LpTokenAmount::add_assign");
        };
        self.0 = result;
    }
}

impl SubAssign for LpTokenAmount {
    fn sub_assign(&mut self, rhs: Self) {
        let Some(result) = self.0.checked_sub(rhs.0) else {
            panic!("Overflow in LpTokenAmount::sub_assign");
        };
        self.0 = result;
    }
}

impl From<LpTokenAmount> for u64 {
    fn from(val: LpTokenAmount) -> Self {
        val.0
    }
}

impl LpTokenAmount {
    pub fn from_tokens_with_fee(amount: TokenAmount, fee: Fee) -> Result<Self> {
        Ok(LpTokenAmount(fee.apply(amount.into())?))
    }

    pub fn from_tokens(amount: TokenAmount) -> Self {
        LpTokenAmount(amount.into())
    }

    pub fn from_lamports(lamports: u64) -> Self {
        LpTokenAmount(lamports)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StakedTokenAmount(u64);

impl Add for StakedTokenAmount {
    type Output = StakedTokenAmount;

    fn add(self, rhs: Self) -> Self::Output {
        let Some(result) = self.0.checked_add(rhs.0) else {
            panic!("Overflow in StakedTokenAmount::add");
        };
        StakedTokenAmount(result)
    }
}

impl AddAssign for StakedTokenAmount {
    fn add_assign(&mut self, rhs: Self) {
        let Some(result) = self.0.checked_add(rhs.0) else {
            panic!("Overflow in StakedTokenAmount::add_assign");
        };
        self.0 = result;
    }
}

impl SubAssign for StakedTokenAmount {
    fn sub_assign(&mut self, rhs: Self) {
        let Some(result) = self.0.checked_sub(rhs.0) else {
            panic!("Overflow in StakedTokenAmount::sub_assign");
        };
        self.0 = result;
    }
}

impl Sub for StakedTokenAmount {
    type Output = StakedTokenAmount;

    fn sub(self, rhs: Self) -> Self::Output {
        let Some(result) = self.0.checked_sub(rhs.0) else {
            panic!("Overflow in StakedTokenAmount::sub");
        };
        StakedTokenAmount(result)
    }
}

impl From<StakedTokenAmount> for u64 {
    fn from(val: StakedTokenAmount) -> Self {
        val.0
    }
}

impl StakedTokenAmount {
    pub fn from_tokens(amount: TokenAmount, price: Price) -> Self {
        StakedTokenAmount(price.div_by_price(amount.into()))
    }

    pub fn from_lamports(lamports: u64) -> Self {
        StakedTokenAmount(lamports)
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn it_creates_token_amount_from_lamports() {
        let token_amount = TokenAmount::from_lamports(10000);
        assert_eq!(token_amount.0, 10000);
    }

    #[test]
    fn it_creates_token_from_staked_tokens() {
        let token_amount =
            TokenAmount::from_staked_tokens(StakedTokenAmount(10000), Price::from_points(2));
        assert_eq!(token_amount.0, 20000);
    }

    #[test]
    fn it_creates_lp_token_from_tokens() {
        let lp_token_amount = LpTokenAmount::from_tokens(TokenAmount(10000));
        assert_eq!(lp_token_amount.0, 10000);
    }

    #[test]
    fn it_creates_lp_token_from_tokens_with_fee() {
        let lp_token_amount =
            LpTokenAmount::from_tokens_with_fee(TokenAmount(10000), Fee::from_basis_points(100))
                .unwrap();
        assert_eq!(lp_token_amount.0, 9900);
    }

    #[test]
    fn it_creates_staked_token_from_tokens() {
        let staked_token_amount =
            StakedTokenAmount::from_tokens(TokenAmount(10000), Price::from_points(2));
        assert_eq!(staked_token_amount.0, 5000);
    }

    #[test]
    fn it_creates_staked_token_from_lamports() {
        let staked_token_amount = StakedTokenAmount::from_lamports(10000);
        assert_eq!(staked_token_amount.0, 10000);
    }
}
