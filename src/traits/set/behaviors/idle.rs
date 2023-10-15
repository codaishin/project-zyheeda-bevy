use crate::{
	behavior::{Behavior, Idle},
	components::Behaviors,
	traits::set::Set,
};

impl Set<Idle> for Behaviors {
	fn set(&mut self, value: Idle) {
		self.0 = vec![Behavior::Idle(value)]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn set() {
		let idle = Idle;
		let mut behaviors = Behaviors(vec![]);

		behaviors.set(idle);

		assert_eq!(vec![Behavior::Idle(idle)], behaviors.0);
	}

	#[test]
	fn override_existing() {
		let idle = Idle;
		let mut behaviors = Behaviors(vec![
			Behavior::Idle(Idle),
			Behavior::Idle(Idle),
			Behavior::Idle(Idle),
		]);

		behaviors.set(idle);

		assert_eq!(vec![Behavior::Idle(idle)], behaviors.0);
	}
}
