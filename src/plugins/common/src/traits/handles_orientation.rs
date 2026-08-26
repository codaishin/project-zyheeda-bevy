use crate::{
	components::persistent_entity::PersistentEntity,
	traits::accessors::get::TryGetContextMut,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use macros::{EntityKey, serde_model};
use std::ops::DerefMut;

pub trait HandlesOrientation {
	type TFaceSystemParam: SystemParam
		+ for<'c> TryGetContextMut<Facing, TContext<'c>: OverrideFace>;
}

#[derive(EntityKey)]
pub struct Facing {
	pub entity: Entity,
}

pub trait OverrideFace {
	fn override_face(&mut self, face: Face);
	fn stop_override_face(&mut self);
}

impl<T> OverrideFace for T
where
	T: DerefMut<Target: OverrideFace>,
{
	fn override_face(&mut self, face: Face) {
		self.deref_mut().override_face(face);
	}

	fn stop_override_face(&mut self) {
		self.deref_mut().stop_override_face();
	}
}

#[serde_model]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum Face {
	#[default]
	SkillTarget,
	Entity(PersistentEntity),
	Translation(Vec3),
	Direction(Dir3),
}
