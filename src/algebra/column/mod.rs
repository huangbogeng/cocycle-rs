//! Ordered sparse coefficient columns, independent of filtration and execution.
use super::PrimeField;
use crate::Result;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct Column<K>(BTreeMap<K, u32>);
impl<K: Ord + Clone> Column<K> {
    pub(crate) fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub(crate) fn unit(key: K) -> Self {
        Self(BTreeMap::from([(key, 1)]))
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub(crate) fn entries(&self) -> impl DoubleEndedIterator<Item = (&K, &u32)> {
        self.0.iter()
    }
    pub(crate) fn first(&self) -> Option<(&K, &u32)> {
        self.0.first_key_value()
    }
    pub(crate) fn last(&self) -> Option<(&K, &u32)> {
        self.0.last_key_value()
    }
    pub(crate) fn get(&self, key: &K) -> u32 {
        self.0.get(key).copied().unwrap_or(0)
    }
    pub(crate) fn add_term(&mut self, key: K, coefficient: u32, field: PrimeField) {
        let value = field.add(self.get(&key), coefficient);
        if value == 0 {
            self.0.remove(&key);
        } else {
            self.0.insert(key, value);
        }
    }
    pub(crate) fn add_scaled(
        &mut self,
        other: &Self,
        factor: u32,
        field: PrimeField,
        checkpoint: &mut impl FnMut() -> Result<()>,
    ) -> Result<()> {
        for (key, &value) in other.entries() {
            checkpoint()?;
            self.add_term(key.clone(), field.multiply(factor, value), field);
        }
        Ok(())
    }
    pub(crate) fn scale(
        &mut self,
        factor: u32,
        field: PrimeField,
        checkpoint: &mut impl FnMut() -> Result<()>,
    ) -> Result<()> {
        for value in self.0.values_mut() {
            checkpoint()?;
            *value = field.multiply(*value, factor);
        }
        self.0.retain(|_, v| *v != 0);
        Ok(())
    }
}
