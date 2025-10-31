use std::ops::DerefMut;

use crate::{
	components::persistent_entity::PersistentEntity,
	traits::accessors::get::EntityContextMut,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};

pub trait HandlesOrientation {
	type TFaceSystemParam<'w, 's>: SystemParam
		+ for<'c> EntityContextMut<Facing, TContext<'c>: OverrideFace + SetFaceTarget>;
}

pub type FacingSystemParam<'w, 's, T> = <T as HandlesOrientation>::TFaceSystemParam<'w, 's>;

pub struct Facing;

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

pub trait SetFaceTarget {
	fn set_face_target(&mut self, target: FaceTargetIs);
}

impl<T> SetFaceTarget for T
where
	T: DerefMut<Target: SetFaceTarget>,
{
	fn set_face_target(&mut self, target: FaceTargetIs) {
		self.deref_mut().set_face_target(target);
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

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum FaceTargetIs {
	Cursor,
	Entity(Entity),
}
