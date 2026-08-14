use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct DefaultAttributes(pub(crate) PhysicalDefaultAttributes);

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
