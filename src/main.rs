use lp_pool::{
    data::{
        fee::Fee,
        price::Price,
        token::{LpTokenAmount, StakedTokenAmount, TokenAmount},
    },
    LpPool,
};

mod calc;
mod error;
mod lp_pool;

fn main() {
    //LpPool::init(price=1.5, min_fee=0.1%, max_fee9%, liquidity_target=90.0 Token)
    let mut lp_pool: LpPool = LpPool::init(
        Price::try_from(1.5).unwrap(),
        Fee::from_basis_points(10),
        Fee::from_basis_points(900),
        TokenAmount::from_lamports(90_000),
    )
    .unwrap();

    lp_pool
        .add_liquidity(TokenAmount::from_lamports(100_000_000))
        .unwrap();

    lp_pool
        .swap(StakedTokenAmount::from_lamports(6_000))
        .unwrap();

    lp_pool
        .add_liquidity(TokenAmount::from_lamports(10_000))
        .unwrap();

    lp_pool
        .remove_liquidity(LpTokenAmount::from_lamports(2_000))
        .unwrap();
}
