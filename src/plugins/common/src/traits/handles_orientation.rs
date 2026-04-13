use crate::{
	components::persistent_entity::PersistentEntity,
	traits::{accessors::get::GetContextMut, handles_skill_physics::Cursor},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use macros::EntityKey;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

pub trait HandlesOrientation {
	type TFaceSystemParam<'w, 's>: SystemParam
		+ for<'c> GetContextMut<Facing, TContext<'c>: OverrideFace>;
}

pub type FacingSystemParamMut<'w, 's, T> = <T as HandlesOrientation>::TFaceSystemParam<'w, 's>;

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

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Face {
	Cursor(Cursor),
	Entity(PersistentEntity),
	Translation(Vec3),
	Direction(Dir3),
}

impl Default for Face {
	fn default() -> Self {
		Self::Cursor(Cursor::default())
	}
}
