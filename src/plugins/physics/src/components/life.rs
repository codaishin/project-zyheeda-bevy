use bevy::prelude::*;
use common::{
	attributes::health::Health,
	tools::attribute::AttributeOnSpawn,
	traits::{
		accessors::get::GetProperty,
		register_derived_component::{DerivableFrom, InsertDerivedComponent},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Life(Health);

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

impl<T> DerivableFrom<'_, '_, T> for Life
where
	T: GetProperty<AttributeOnSpawn<Health>>,
{
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;

	type TParam = ();

	fn derive_from(_: Entity, component: &T, _: &()) -> Self {
		Life(component.get_property())
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
