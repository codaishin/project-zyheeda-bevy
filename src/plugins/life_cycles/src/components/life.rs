use bevy::prelude::*;
use common::{
	attributes::health::Health,
	traits::{accessors::get::GetterRef, handles_life::ChangeLife},
};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Life(pub(crate) Health);

impl ChangeLife for Life {
	fn change_by(&mut self, value: f32) {
		let Life(Health { current, max }) = self;

		*current += value;
		*current = current.min(*max);
	}
}

impl From<Health> for Life {
	fn from(health: Health) -> Self {
		Life(health)
	}
}

impl GetterRef<Health> for Life {
	fn get(&self) -> &Health {
		let Life(health) = self;

		health
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn update_life() {
		let mut life = Life(Health {
			current: 42.,
			max: 100.,
		});

		life.change_by(11.);

		assert_eq!(
			Life(Health {
				current: 53.,
				max: 100.,
			}),
			life
		);
	}

	#[test]
	fn do_not_surpass_max() {
		let mut life = Life(Health {
			current: 87.,
			max: 100.,
		});

		life.change_by(101.);

		assert_eq!(
			Life(Health {
				current: 100.,
				max: 100.,
			}),
			life
		);
	}
}
