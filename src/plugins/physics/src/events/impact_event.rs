use bevy::prelude::*;
use common::prelude::*;

#[derive(EntityEvent, Debug, PartialEq)]
pub struct ImpactEvent {
	pub(crate) entity: Entity,
	pub(crate) position: VecNotNan<3>,
}

impl View<GlobalVec3> for ImpactEvent {
	fn view(&self) -> &'_ VecNotNan<3> {
		&self.position
	}
}
