use common::traits::iteration::{Infinite, IterInfinite};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct InventoryKey(pub usize);

impl IterInfinite for InventoryKey {
	fn iterator_infinite() -> Infinite<Self> {
		Infinite(InventoryKey(0))
	}

	fn next_infinite(current: &Infinite<Self>) -> Self {
		InventoryKey(current.0 .0 + 1)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn keys() {
		let keys = InventoryKey::iterator_infinite();

		assert_eq!(
			vec![0, 1, 2, 3, 4],
			keys.map(|i| i.0).take(5).collect::<Vec<_>>()
		);
	}
}
