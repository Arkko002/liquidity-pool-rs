use crate::error::{Error, Result};
use crate::lp_pool::error::Error as LpPoolError;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Price(u64);

impl TryFrom<f32> for Price {
    type Error = Error;
    fn try_from(price_without_scale: f32) -> Result<Self> {
        #[allow(clippy::cast_possible_truncation)]
        let price_i = (price_without_scale * 100.0).floor() as i64;
        let price = u32::try_from(price_i).map_err(|_| {
            Error::LpPool(LpPoolError::PriceConversionFailure {
                converted_from: price_without_scale.to_string(),
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
    pub fn from_points(points: u64) -> Self {
        Self(points)
    }

    pub fn mul_by_price(self, lamports: u64) -> u64 {
        u64::try_from(u128::from(lamports) * u128::from(self.0))
            .expect("Overflow in Price::mul_by_price")
    }

    pub fn div_by_price(self, lamports: u64) -> u64 {
        u64::try_from(u128::from(lamports) / u128::from(self.0))
            .expect("Overflow in Price::div_by_price")
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn it_converts_price_in_float() {
        let price = Price::try_from(0.01).unwrap();
        assert_eq!(price.0, 1);
    }

    #[test]
    fn it_converts_price_in_u64() {
        let price = Price::try_from(100).unwrap();
        assert_eq!(price.0, 10000);
    }
}
