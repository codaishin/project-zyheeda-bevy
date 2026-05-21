mod mouse_hover;
mod mouse_terrain_hover;
mod solid_objects;
mod terrain;

use crate::components::{collider::ChildColliderOf, offset::AimOffset, world_camera::WorldCamera};
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
	child_colliders: Query<'w, 's, &'static ChildColliderOf>,
	transforms: Query<'w, 's, (&'static GlobalTransform, Option<&'static AimOffset>)>,
	world_cams: Query<'w, 's, &'static mut WorldCamera>,
}
