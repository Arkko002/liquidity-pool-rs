use derive_more::Display;
use derive_more::From;

use crate::lp_pool::error::Error as LpPoolError;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    LpPool(LpPoolError),

    CalculationError,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
