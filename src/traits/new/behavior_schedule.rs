use crate::components::BehaviorSchedule;
use crate::traits::new::New;

impl New for BehaviorSchedule {
	fn new() -> Self {
		BehaviorSchedule(vec![])
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn create_new() {
		let schedule = BehaviorSchedule::new();
		assert!(schedule.0.is_empty());
	}
}
