use super::*;
use std::{any::type_name, collections::VecDeque};

/// Holds unique values, but retains insertion order.
///
/// Removal and Insertion are `O(n)` operations.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub(super) struct Unique<TKey>(VecDeque<TKey>)
where
	TKey: PartialEq;

impl<TKey> Unique<TKey>
where
	TKey: PartialEq,
{
	pub(super) const EMPTY: Self = Self(VecDeque::new());

	#[cfg(test)]
	fn from_unchecked(values: impl Into<VecDeque<TKey>>) -> Self {
		Self(values.into())
	}

	pub(super) fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub(super) fn push_back_unique(&mut self, key: TKey) {
		self.remove(&key);
		self.0.push_back(key);
	}

	pub(super) fn remove(&mut self, key: &TKey) {
		// It is enough to find the first hit, because we run this before each insertion.
		// There is always just one matching item contained.
		let Some(i) = self.0.iter().position(|k| k == key) else {
			return;
		};

		self.0.remove(i);
	}

	pub(super) fn clear(&mut self) {
		self.0.clear();
	}

	pub(super) fn pop_front(&mut self) -> Option<TKey> {
		self.0.pop_front()
	}

	pub(super) fn iter(&self) -> super::UniqueIter<'_, TKey> {
		self.0.iter()
	}
}

impl<TKey> Default for Unique<TKey>
where
	TKey: PartialEq,
{
	fn default() -> Self {
		Self(VecDeque::default())
	}
}

impl<'de, TKey> Deserialize<'de> for Unique<TKey>
where
	TKey: PartialEq + Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let keys = VecDeque::deserialize(deserializer)?;
		let mut seen = vec![];

		for key in &keys {
			if seen.contains(&key) {
				return Err(serde::de::Error::custom(format!(
					"{}: encountered duplicate keys",
					type_name::<Self>()
				)));
			}

			seen.push(key);
		}

		Ok(Self(keys))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::{Error, json};

	#[test]
	fn deserialize() -> Result<(), Error> {
		let json = json!([1, 2, 3]);

		let keys = serde_json::from_value::<Unique<u8>>(json)?;

		assert_eq!(Unique::<u8>::from_unchecked([1, 2, 3]), keys);
		Ok(())
	}

	#[test]
	fn deserialize_rejection_on_non_unique() {
		let json = json!([1, 2, 2, 3]);

		let keys = serde_json::from_value::<Unique<u8>>(json);

		assert!(keys.is_err());
	}
}
