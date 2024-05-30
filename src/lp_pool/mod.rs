pub mod data;
pub mod error;

use crate::lp_pool::data::{
    fee::Fee,
    token::{LpTokenAmount, StakedTokenAmount, TokenAmount},
};

use crate::{
    calc::proportional,
    error::{Error, Result},
};
use crate::{calc::value_from_shares, lp_pool::error::Error as LpPoolError};

use self::data::price::Price;

pub struct LpPool {
    price: Price,
    token_amount: TokenAmount,
    staked_token_amount: StakedTokenAmount,
    lp_token_amount: LpTokenAmount,
    liquidity_target: TokenAmount,
    min_fee: Fee,
    max_fee: Fee,
}

impl LpPool {
    pub fn init(
        price: Price,
        min_fee: Fee,
        max_fee: Fee,
        liquidity_target: TokenAmount,
    ) -> Result<Self> {
        if min_fee > max_fee {
            return Err(Error::LpPool(LpPoolError::MinFeeGreaterThanMaxFee {
                min: min_fee,
                max: max_fee,
            }));
        }

        if liquidity_target == TokenAmount::from_lamports(0) {
            return Err(Error::LpPool(LpPoolError::LiquidityTargetIncorrect(
                liquidity_target,
            )));
        }

        if price == Price::try_from(0)? {
            return Err(Error::LpPool(LpPoolError::PriceIncorrect(price)));
        }

        Ok(Self {
            price,
            min_fee,
            max_fee,
            liquidity_target,
            token_amount: TokenAmount::from_lamports(0),
            staked_token_amount: StakedTokenAmount::from_lamports(0),
            lp_token_amount: LpTokenAmount::from_lamports(0),
        })
    }

    pub fn add_liquidity(&mut self, tokens_to_add: TokenAmount) -> Result<LpTokenAmount> {
        let token_amount_after =
            TokenAmount::from_lamports(u64::from(self.token_amount) + u64::from(tokens_to_add));
        let fee: Fee = self.calculate_fee(token_amount_after);
        let tokens_with_fee = TokenAmount::from_lamports(fee.apply(tokens_to_add.into())?);

        self.token_amount += tokens_with_fee;
        self.staked_token_amount += StakedTokenAmount::from_tokens(tokens_with_fee, self.price);
        self.lp_token_amount += LpTokenAmount::from_tokens(tokens_with_fee);
        Ok(LpTokenAmount::from_tokens(tokens_with_fee))
    }

    pub fn remove_liquidity(
        &mut self,
        lp_tokens_to_remove: LpTokenAmount,
    ) -> Result<(TokenAmount, StakedTokenAmount)> {
        let tokens_without_fee = TokenAmount::from_lamports(value_from_shares(
            lp_tokens_to_remove.into(),
            self.token_amount.into(),
            self.lp_token_amount.into(),
        )?);

        let token_amount_after = TokenAmount::from_lamports(
            u64::from(self.token_amount) - u64::from(tokens_without_fee),
        );
        let fee: Fee = self.calculate_fee(token_amount_after);

        let tokens_with_fee: TokenAmount =
            TokenAmount::from_lamports(fee.apply(tokens_without_fee.into())?);
        let unstaked_tokens: StakedTokenAmount =
            StakedTokenAmount::from_tokens(tokens_with_fee, self.price);

        self.token_amount -= tokens_with_fee;
        self.staked_token_amount -= unstaked_tokens;
        self.lp_token_amount -= lp_tokens_to_remove;

        Ok((tokens_with_fee, unstaked_tokens))
    }

    pub fn swap(&mut self, staked_tokens_to_swap: StakedTokenAmount) -> Result<TokenAmount> {
        let token_amount_after = TokenAmount::from_lamports(
            u64::from(self.token_amount)
                - self.price.mul_by_price(u64::from(staked_tokens_to_swap)),
        );
        let fee: Fee = self.calculate_fee(token_amount_after);

        let tokens_without_fee = TokenAmount::from_staked_tokens(staked_tokens_to_swap, self.price);
        let tokens_with_fee = TokenAmount::from_lamports(fee.apply(tokens_without_fee.into())?);

        self.token_amount -= tokens_with_fee;
        self.staked_token_amount += staked_tokens_to_swap;

        Ok(tokens_with_fee)
    }

    pub fn calculate_fee(&self, amount_after: TokenAmount) -> Fee {
        if amount_after >= self.liquidity_target {
            return Fee::from_basis_points(self.min_fee.basis_points);
        }

        let fee_delta: u64 = u64::from(
            self.max_fee
                .basis_points
                .saturating_sub(self.min_fee.basis_points),
        );
        let lamports: u64 = amount_after.into();
        let target: u64 = self.liquidity_target.into();

        #[allow(clippy::cast_possible_truncation)]
        Fee::from_basis_points(
            self.max_fee.basis_points - proportional(fee_delta, lamports, target).unwrap() as u32,
        )
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn it_returns_err_if_min_fee_greater_than_max_fee() {
        let lp_pool = LpPool::init(
            Price::try_from(10).unwrap(),
            Fee::from_basis_points(101),
            Fee::from_basis_points(100),
            TokenAmount::from_lamports(100),
        );
        assert!(lp_pool.is_err());
    }

    #[test]
    fn it_returns_err_if_liquidity_target_is_zero() {
        let lp_pool = LpPool::init(
            Price::try_from(10).unwrap(),
            Fee::from_basis_points(10),
            Fee::from_basis_points(100),
            TokenAmount::from_lamports(0),
        );
        assert!(lp_pool.is_err());
    }

    #[test]
    fn it_returns_err_if_price_is_zero() {
        let lp_pool = LpPool::init(
            Price::from_points(0),
            Fee::from_basis_points(10),
            Fee::from_basis_points(100),
            TokenAmount::from_lamports(100),
        );
        assert!(lp_pool.is_err());
    }

    #[test]
    fn it_adds_liquidity_to_pool_and_returns_lp_tokens() {
        let mut lp_pool = LpPool::init(
            Price::try_from(10).unwrap(),
            Fee::from_basis_points(10),
            Fee::from_basis_points(100),
            TokenAmount::from_lamports(100),
        )
        .unwrap();
        let predicted_tokens: LpTokenAmount =
            LpTokenAmount::from_tokens(TokenAmount::from_lamports(100));
        let tokens: LpTokenAmount = lp_pool
            .add_liquidity(TokenAmount::from_lamports(100))
            .unwrap();

        assert_eq!(tokens, predicted_tokens);
        assert_eq!(lp_pool.token_amount, TokenAmount::from_lamports(100));
        assert_eq!(lp_pool.lp_token_amount, predicted_tokens);
        assert_eq!(
            lp_pool.staked_token_amount,
            StakedTokenAmount::from_tokens(TokenAmount::from_lamports(100), lp_pool.price)
        );
    }

    #[test]
    fn it_removes_liquidity_from_pool_above_liquidity_target() {
        let mut lp_pool = LpPool::init(
            Price::from_points(100),
            Fee::from_basis_points(10),
            Fee::from_basis_points(100),
            TokenAmount::from_lamports(50),
        )
        .unwrap();
        lp_pool
            .add_liquidity(TokenAmount::from_lamports(100))
            .unwrap();
        let predicted_tokens = TokenAmount::from_lamports(lp_pool.min_fee.apply(10).unwrap());

        let (tokens, staked_tokens) = lp_pool
            .remove_liquidity(LpTokenAmount::from_lamports(10))
            .unwrap();

        assert_eq!(tokens, predicted_tokens);
        assert_eq!(
            staked_tokens,
            StakedTokenAmount::from_tokens(predicted_tokens, lp_pool.price)
        );
        assert_eq!(
            lp_pool.token_amount,
            (TokenAmount::from_lamports(100) - predicted_tokens)
        );
        assert_eq!(lp_pool.lp_token_amount, LpTokenAmount::from_lamports(90));
        assert_eq!(
            lp_pool.staked_token_amount,
            (StakedTokenAmount::from_tokens(TokenAmount::from_lamports(100), lp_pool.price)
                - StakedTokenAmount::from_tokens(predicted_tokens, lp_pool.price))
        );
    }

    #[test]
    fn it_swaps_tokens_based_on_price() {
        let mut lp_pool = LpPool::init(
            Price::from_points(2),
            Fee::from_basis_points(100),
            Fee::from_basis_points(1000),
            TokenAmount::from_lamports(10),
        )
        .unwrap();
        lp_pool
            .add_liquidity(TokenAmount::from_lamports(200))
            .unwrap();

        let tokens: TokenAmount = lp_pool.swap(StakedTokenAmount::from_lamports(50)).unwrap();
        assert_eq!(tokens, TokenAmount::from_lamports(99));
        assert_eq!(lp_pool.token_amount, TokenAmount::from_lamports(99));
        assert_eq!(
            lp_pool.staked_token_amount,
            StakedTokenAmount::from_lamports(149)
        );
    }
}
