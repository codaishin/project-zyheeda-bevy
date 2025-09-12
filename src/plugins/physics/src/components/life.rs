use bevy::prelude::*;
use common::attributes::health::Health;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Life(pub(crate) Health);

impl Life {
	pub fn change_by(&mut self, health: f32) {
		let Life(Health { current, max }) = self;

		*current += health;
		*current = current.min(*max);
	}
}

impl From<&Life> for Health {
	fn from(Life(health): &Life) -> Self {
		*health
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
