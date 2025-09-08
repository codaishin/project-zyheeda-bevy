#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(super) struct InteractionsCount(usize);

impl InteractionsCount {
	pub(super) fn one() -> Self {
		InteractionsCount(1)
	}

	pub(super) fn increment(&mut self) {
		self.0 += 1;
	}

	pub(super) fn try_decrement(self) -> RemainingInteractions {
		if self.0 > 1 {
			RemainingInteractions::Some(Self(self.0 - 1))
		} else {
			RemainingInteractions::None
		}
	}
}

#[derive(Debug, PartialEq)]
pub(super) enum RemainingInteractions {
	Some(InteractionsCount),
	None,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn init_count() {
		let count = InteractionsCount::one();

		assert_eq!(InteractionsCount(1), count);
	}

	#[test]
	fn increment() {
		let mut count = InteractionsCount::one();

		count.increment();

		assert_eq!(InteractionsCount(2), count);
	}

	#[test]
	fn decrement_none() {
		let count = InteractionsCount::one();

		let count = count.try_decrement();

		assert_eq!(RemainingInteractions::None, count);
	}

	#[test]
	fn decrement_some() {
		let mut count = InteractionsCount::one();

		count.increment();
		let count = count.try_decrement();

		assert_eq!(RemainingInteractions::Some(InteractionsCount(1)), count);
	}
}
