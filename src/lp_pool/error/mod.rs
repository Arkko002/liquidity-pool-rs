use super::data::{
    fee::Fee,
    token::{Price, TokenAmount},
};

#[derive(Debug)]
pub enum Error {
    LiquidityTargetIncorrect(TokenAmount),
    CalculationError { message: String },
    PriceIncorrect(Price),
    PriceConversionFailure { converted_from: String },
    BasisPointsOverflow(u32),
    MinFeeGreaterThanMaxFee { min: Fee, max: Fee },
}

// TODO: Meaningful error messages
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BasisPointsOverflow(basis) => write!(f, "BasisPointsOverflow({})", basis),
            Error::MinFeeGreaterThanMaxFee { min, max } => {
                write!(f, "MinFeeGreaterThanMaxFee(min: {}, max: {})", min, max)
            }
            Error::LiquidityTargetIncorrect(liquidity_target) => {
                write!(f, "IncorrectLiquidityTarget({})", liquidity_target)
            }
            Error::PriceIncorrect(price) => write!(f, "IncorrectPrice({})", price),
            Error::PriceConversionFailure { converted_from } => {
                write!(
                    f,
                    "PriceConversionError(converted_from: {})",
                    converted_from
                )
            }
            Error::CalculationError { message } => write!(f, "CalculationError({})", message),
        }
    }
}

impl std::error::Error for Error {}
