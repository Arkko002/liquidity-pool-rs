use super::data::{
    fee::Fee,
    token::{LpTokenAmount, Price, StakedTokenAmount, TokenAmount},
};

use crate::error::{Error, Result};
use crate::lp_pool::error::Error as LpPoolError;

// NOTE: Price of mSOL = total_staked / tokens_minted
// NOTE: https://github.com/marinade-finance/liquid-staking-program
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
        let token_amount_after: TokenAmount = self.token_amount + tokens_to_add;
        let fee: Fee = self.calculate_fee(token_amount_after);

        self.token_amount += tokens_to_add;
        self.lp_token_amount += LpTokenAmount::from_tokens_flat(tokens_to_add);
        return Ok(LpTokenAmount::from_tokens_with_fee(
            tokens_to_add,
            self.min_fee,
        ));
    }
    pub fn remove_liquidity(
        &mut self,
        lp_tokens_to_remove: LpTokenAmount,
    ) -> Result<(TokenAmount, StakedTokenAmount)> {
        Ok((TokenAmount(0), StakedTokenAmount(0)))
    }
    pub fn swap(&mut self, staked_tokens_to_swap: StakedTokenAmount) -> Result<TokenAmount> {
        // TODO: liquidity_target
        Ok(TokenAmount(staked_tokens_to_swap.0 * self.price.0))
    }

    // NOTE: Formula source: https://docs.marinade.finance/marinade-protocol/system-overview/unstake-liquidity-pool#fee-calculation
    pub fn calculate_fee(&self, amount_after: TokenAmount) -> Fee {
        // unstake_fee = max_fee - (max_fee - min_fee) * amount_after / target
        if amount_after >= self.liquidity_target {
            return Fee::from_basis_points(self.min_fee.basis_points);
        }

        let fee_delta: u64 = self
            .max_fee
            .basis_points
            .saturating_sub(self.min_fee.basis_points) as u64;
        let lamports: u64 = amount_after.into();
        let target: u64 = self.liquidity_target.into();

        Fee::from_basis_points(
            self.max_fee.basis_points - proportional(fee_delta, lamports, target).unwrap() as u32,
        )
    }
}

// NOTE: Formula-soruce: liquid-staking-program/programs/marinade-finance/src/calc.rs
pub fn proportional(amount: u64, numerator: u64, denominator: u64) -> Result<u64> {
    if denominator == 0 {
        return Ok(amount);
    }
    u64::try_from((amount as u128) * (numerator as u128) / (denominator as u128)).map_err(|_| {
        Error::LpPool(LpPoolError::CalculationError {
            message: format!(
                "Failed calculating proportions: {amount} * {numerator} / {denominator}"
            ),
        })
    })
}

mod tests {
    use super::*;

    #[test]
    fn it_creates_a_lp_pool_with_valid_params() {
        let lp_pool = LpPool::init(
            Price::try_from(10).unwrap(),
            Fee::from_basis_points(10),
            Fee::from_basis_points(100),
            TokenAmount::from_lamports(100),
        )
        .unwrap();
        assert_eq!(lp_pool.token_amount.0, 0);
        assert_eq!(lp_pool.staked_token_amount.0, 0);
        assert_eq!(lp_pool.lp_token_amount.0, 0);
        assert_eq!(lp_pool.liquidity_target.0, 100);
    }

    #[test]
    fn it_returns_err_if_min_fee_greater_than_max_fee() {
        let lp_pool = LpPool::init(
            Price::try_from(10).unwrap(),
            Fee::from_basis_points(10),
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
            TokenAmount::from_lamports(100),
        );
        assert!(lp_pool.is_err());
    }

    #[test]
    fn it_returns_err_if_price_is_zero() {
        let lp_pool = LpPool::init(
            Price::try_from(10).unwrap(),
            Fee::from_basis_points(10),
            Fee::from_basis_points(100),
            TokenAmount::from_lamports(100),
        );
        assert!(lp_pool.is_err());
    }

    // TODO: Check lp_amount, token_amount values etc.
    #[test]
    fn it_adds_liquidity_to_pool_above_liquidity_target() {
        let mut lp_pool = LpPool::init(
            Price::try_from(10).unwrap(),
            Fee::from_basis_points(10),
            Fee::from_basis_points(100),
            TokenAmount::from_lamports(100),
        )
        .unwrap();
        let tokens: LpTokenAmount = lp_pool.add_liquidity(TokenAmount(100)).unwrap();

        assert_eq!(
            tokens.0,
            Fee::from_basis_points(10).apply(TokenAmount(100).0)
        )
    }

    #[test]
    fn it_adds_liquidity_to_pool_below_liquidity_target() {
        let mut lp_pool = LpPool::init(
            Price::try_from(10).unwrap(),
            Fee::from_basis_points(10),
            Fee::from_basis_points(100),
            TokenAmount::from_lamports(100),
        )
        .unwrap();
        let fee: Fee = lp_pool.calculate_fee(TokenAmount(100));
        let tokens: LpTokenAmount = lp_pool.add_liquidity(TokenAmount(100)).unwrap();

        assert_eq!(tokens.0, fee.apply(TokenAmount(100).0))
    }

    #[test]
    fn it_removes_liquidity_from_pool_above_liquidity_target() {
        let mut lp_pool = LpPool::init(
            Price(10000),
            Fee::from_basis_points(10),
            Fee::from_basis_points(100),
            TokenAmount(100),
        )
        .unwrap();
        lp_pool.add_liquidity(TokenAmount(120)).unwrap();
        let _ = lp_pool.remove_liquidity(LpTokenAmount(10)).unwrap();
    }

    #[test]
    fn it_swaps_tokens_based_on_price() {
        let mut lp_pool = LpPool::init(
            Price(10000),
            Fee::from_basis_points(10),
            Fee::from_basis_points(100),
            TokenAmount(100),
        )
        .unwrap();

        // TODO:
        let tokens: TokenAmount = lp_pool.swap(StakedTokenAmount(100)).unwrap();
        assert_eq!(tokens.0, 10000);
    }
}
