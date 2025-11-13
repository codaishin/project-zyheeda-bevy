use crate::traits::animation::AnimationPriority;

#[derive(Debug, PartialEq)]
pub struct DescendingAnimationPriorities {
	current: Option<AnimationPriority>,
}

impl Default for DescendingAnimationPriorities {
	fn default() -> Self {
		Self {
			current: Some(AnimationPriority::High),
		}
	}
}

impl Iterator for DescendingAnimationPriorities {
	type Item = AnimationPriority;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.current;

		self.current = match next? {
			AnimationPriority::High => Some(AnimationPriority::Medium),
			AnimationPriority::Medium => Some(AnimationPriority::Low),
			AnimationPriority::Low => None,
		};

		next
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn order() {
		assert_eq!(
			vec![
				AnimationPriority::High,
				AnimationPriority::Medium,
				AnimationPriority::Low,
			],
			AnimationPriority::ordered_descending()
				.take(100)
				.collect::<Vec<_>>()
		);
	}
}
