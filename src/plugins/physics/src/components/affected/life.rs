use crate::systems::insert_affected::AffectedComponent;
use bevy::prelude::*;
use common::{attributes::health::Health, traits::accessors::get::GetProperty};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[savable_component(id = "life")]
pub struct Life(pub(crate) Health);

impl Life {
	pub(crate) fn change_by(&mut self, health: f32) {
		let Life(Health { current, max }) = self;

		*current += health;
		*current = current.min(*max);
	}

	pub(crate) fn current_hp(&self) -> f32 {
		self.0.current
	}
}

impl From<Health> for Life {
	fn from(health: Health) -> Self {
		Self(health)
	}
}

impl GetProperty<Health> for Life {
	fn get_property(&self) -> Health {
		self.0
	}
}

impl AffectedComponent for Life {
	type TAttribute = Health;
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
