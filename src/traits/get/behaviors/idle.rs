use crate::{
	behavior::{Behavior, Idle},
	components::Behaviors,
	traits::get::Get,
};

impl Get<Idle> for Behaviors {
	fn get(&self) -> Option<Idle> {
		match self.0.first() {
			Some(Behavior::Idle(idle)) => Some(*idle),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::{new::New, set::Set};

	#[test]
	fn get_none() {
		let mut scheduler = Behaviors::new();

		assert!((&mut scheduler as &mut dyn Get<Idle>).get().is_none());
	}

	#[test]
	fn get_first() {
		let mut scheduler = Behaviors::new();
		let idle = Idle;

		(&mut scheduler as &mut dyn Set<Idle>).set(idle);

		assert_eq!(idle, scheduler.get().unwrap());
	}
}
