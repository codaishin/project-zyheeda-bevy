use bevy::prelude::*;
use common::{components::Health, traits::handles_life::ChangeLife};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct Life(pub(crate) Health);

impl ChangeLife for Life {
	fn change_by(&mut self, value: f32) {
		let Life(Health { current, max }) = self;

		*current += value;
		*current = current.min(*max);
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
