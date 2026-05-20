mod mouse_hover;
mod mouse_terrain_hover;
mod solid_objects;
mod terrain;

use crate::components::{
	collider::ChildCollider,
	effect_target::EffectTarget,
	offset::AimOffset,
	world_camera::WorldCamera,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use bevy_rapier3d::prelude::*;

#[derive(SystemParam)]
pub struct RayCaster<'w, 's, T = ReadRapierContext<'static, 'static>>
where
	T: SystemParam + 'static,
{
	context: StaticSystemParam<'w, 's, T>,
	effect_target_child_colliders: Query<'w, 's, &'static ChildCollider<EffectTarget>>,
	rigid_body_child_colliders: Query<'w, 's, &'static ChildCollider<RigidBody>>,
	transforms: Query<'w, 's, (&'static GlobalTransform, Option<&'static AimOffset>)>,
	world_cams: Query<'w, 's, &'static mut WorldCamera>,
}
