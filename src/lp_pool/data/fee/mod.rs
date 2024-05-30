use std::fmt::Display;

use crate::error::{Error, Result};
use crate::lp_pool::error::Error as LpPoolError;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Fee {
    pub basis_points: u32,
}

impl Display for Fee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{:0>2}%",
            self.basis_points / 100,
            self.basis_points % 100
        )
    }
}

impl Fee {
    pub const MAX_BASIS_POINTS: u32 = 10_000; //100%

    pub const fn from_basis_points(basis_points: u32) -> Self {
        Self { basis_points }
    }

    pub fn check(self) -> Result<()> {
        if self.basis_points > Self::MAX_BASIS_POINTS {
            return Err(Error::LpPool(LpPoolError::BasisPointsOverflow(
                self.basis_points,
            )));
        }
        Ok(())
    }

    pub fn apply(self, lamports: u64) -> Result<u64> {
        let fee = u64::try_from(
            u128::from(lamports) * u128::from(self.basis_points)
                / u128::from(Self::MAX_BASIS_POINTS),
        )
        .map_err(|_| Error::CalculationError)?;
        Ok(lamports - fee)
    }
}

#[cfg(test)]
mod fee_tests {
    use super::*;
    #[test]
    fn it_creates_fee_with_valid_basis_points() {
        let fee = Fee { basis_points: 10 };
        assert!(fee.check().is_ok());
        assert_eq!(fee.apply(10000).unwrap(), 9990);
    }

    #[test]
    fn it_returns_err_if_basis_points_overflow() {
        let fee = Fee {
            basis_points: 10001,
        };
        assert!(fee.check().is_err());
    }
}
