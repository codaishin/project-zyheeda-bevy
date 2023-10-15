use crate::{components::Behaviors, traits::new::New};

impl New for Behaviors {
	fn new() -> Self {
		Behaviors(vec![])
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn create_new() {
		let schedule = Behaviors::new();
		assert!(schedule.0.is_empty());
	}
}
