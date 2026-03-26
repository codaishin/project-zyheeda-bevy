use bevy::prelude::*;
use common::{
	attributes::{effect_target::EffectTarget, health::Health},
	effects::{force::Force, gravity::Gravity},
	tools::attribute::AttributeOnSpawn,
	traits::{accessors::get::View, handles_physics::PhysicalDefaultAttributes},
};

#[derive(Component, Debug, PartialEq)]
pub struct DefaultAttributes(PhysicalDefaultAttributes);

impl From<PhysicalDefaultAttributes> for DefaultAttributes {
	fn from(attributes: PhysicalDefaultAttributes) -> Self {
		Self(attributes)
	}
}

impl View<AttributeOnSpawn<Health>> for DefaultAttributes {
	fn view(&self) -> Health {
		self.0.health
	}
}

impl View<AttributeOnSpawn<EffectTarget<Gravity>>> for DefaultAttributes {
	fn view(&self) -> EffectTarget<Gravity> {
		self.0.gravity_interaction
	}
}

impl View<AttributeOnSpawn<EffectTarget<Force>>> for DefaultAttributes {
	fn view(&self) -> EffectTarget<Force> {
		self.0.force_interaction
	}
}
