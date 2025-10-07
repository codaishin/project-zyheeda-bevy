use bevy::prelude::*;
use common::{
	attributes::{effect_target::EffectTarget, health::Health},
	effects::{force::Force, gravity::Gravity},
	tools::attribute::AttributeOnSpawn,
	traits::{accessors::get::GetProperty, handles_physics::PhysicalDefaultAttributes},
};

#[derive(Component, Debug, PartialEq)]
pub struct DefaultAttributes(PhysicalDefaultAttributes);

impl From<PhysicalDefaultAttributes> for DefaultAttributes {
	fn from(attributes: PhysicalDefaultAttributes) -> Self {
		Self(attributes)
	}
}

impl GetProperty<AttributeOnSpawn<Health>> for DefaultAttributes {
	fn get_property(&self) -> Health {
		self.0.health
	}
}

impl GetProperty<AttributeOnSpawn<EffectTarget<Gravity>>> for DefaultAttributes {
	fn get_property(&self) -> EffectTarget<Gravity> {
		self.0.gravity_interaction
	}
}

impl GetProperty<AttributeOnSpawn<EffectTarget<Force>>> for DefaultAttributes {
	fn get_property(&self) -> EffectTarget<Force> {
		self.0.force_interaction
	}
}
