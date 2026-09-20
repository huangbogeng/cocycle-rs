//! Exact arithmetic in prime fields with u32 characteristic.
use crate::{Error, Result};

/// A validated prime field Fp for any prime `u32` characteristic.
///
/// Arithmetic accepts arbitrary `u32` residues and returns canonical values in
/// `0..p`. Products use `u64`, including at the largest supported primes. The
/// default is F2. Construction uses exact trial division through sqrt(p).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimeField {
    characteristic: u32,
}
impl PrimeField {
    /// Validate a prime characteristic; zero, one and composites are rejected.
    ///
    /// # Errors
    /// Returns [`Error::InvalidParameter`] for a non-prime characteristic.
    pub fn new(characteristic: u32) -> Result<Self> {
        if characteristic < 2 || (characteristic != 2 && characteristic.is_multiple_of(2)) {
            return Err(invalid());
        }
        let mut divisor = 3;
        while divisor <= characteristic / divisor {
            if characteristic.is_multiple_of(divisor) {
                return Err(invalid());
            }
            divisor += 2;
        }
        Ok(Self { characteristic })
    }
    /// Prime characteristic of this field.
    pub fn characteristic(self) -> u32 {
        self.characteristic
    }
    /// Canonical residue of an unsigned integer.
    pub fn reduce(self, value: u32) -> u32 {
        value % self.characteristic
    }
    /// Sum modulo the characteristic, without intermediate overflow.
    pub fn add(self, left: u32, right: u32) -> u32 {
        ((u64::from(left) + u64::from(right)) % u64::from(self.characteristic)) as u32
    }
    /// Product modulo the characteristic, without intermediate overflow.
    pub fn multiply(self, left: u32, right: u32) -> u32 {
        (u64::from(left) * u64::from(right) % u64::from(self.characteristic)) as u32
    }
    /// Additive inverse, with zero mapped to zero.
    pub fn negate(self, value: u32) -> u32 {
        let value = self.reduce(value);
        if value == 0 {
            0
        } else {
            self.characteristic - value
        }
    }
    /// Difference modulo the characteristic.
    pub fn subtract(self, left: u32, right: u32) -> u32 {
        self.add(left, self.negate(right))
    }
    /// Multiplicative inverse by modular exponentiation.
    ///
    /// # Errors
    /// Returns [`Error::InvalidParameter`] if the input reduces to zero.
    pub fn inverse(self, value: u32) -> Result<u32> {
        let mut base = self.reduce(value);
        if base == 0 {
            return Err(Error::InvalidParameter {
                parameter: "field element",
                reason: "zero has no multiplicative inverse",
            });
        }
        let mut power = self.characteristic - 2;
        let mut result = 1;
        while power != 0 {
            if power & 1 != 0 {
                result = self.multiply(result, base);
            }
            base = self.multiply(base, base);
            power >>= 1;
        }
        Ok(result)
    }
    pub(crate) fn orientation(self, omitted: usize) -> u32 {
        if omitted.is_multiple_of(2) {
            1
        } else {
            self.characteristic - 1
        }
    }
}
impl Default for PrimeField {
    fn default() -> Self {
        Self { characteristic: 2 }
    }
}
fn invalid() -> Error {
    Error::InvalidParameter {
        parameter: "characteristic",
        reason: "must be a prime u32",
    }
}
