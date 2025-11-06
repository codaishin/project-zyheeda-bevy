use crate::{
	components::persistent_entity::PersistentEntity,
	traits::accessors::get::GetContextMut,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

pub trait HandlesOrientation {
	type TFaceSystemParam<'w, 's>: SystemParam
		+ for<'c> GetContextMut<Facing, TContext<'c>: OverrideFace + RegisterFaceTargetDefinition>;
}

pub type FacingSystemParamMut<'w, 's, T> = <T as HandlesOrientation>::TFaceSystemParam<'w, 's>;

pub struct Facing {
	pub entity: Entity,
}

impl From<Facing> for Entity {
	fn from(Facing { entity }: Facing) -> Self {
		entity
	}
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

pub trait RegisterFaceTargetDefinition {
	fn register(&mut self, face_target_is: FaceTargetIs);
}

impl<T> RegisterFaceTargetDefinition for T
where
	T: DerefMut<Target: RegisterFaceTargetDefinition>,
{
	fn register(&mut self, face_target_is: FaceTargetIs) {
		self.deref_mut().register(face_target_is);
	}
}

#[derive(Default, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Face {
	#[default]
	/// This dependents on [`FaceTargetIs`] for the corresponding entity
	Target,
	Entity(PersistentEntity),
	Translation(Vec3),
	Direction(Dir3),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FaceTargetIs {
	Cursor,
	Entity(Entity),
}
