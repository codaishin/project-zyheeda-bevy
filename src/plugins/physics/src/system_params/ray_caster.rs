mod mouse_hover;
mod mouse_terrain_hover;
mod solid_objects;
mod terrain;
mod update_target_ray;

use crate::{
	components::{collider::ColliderOf, offset::AimOffset},
	resources::world_camera::WorldCamera,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use bevy_rapier3d::prelude::*;

#[derive(SystemParam)]

pub struct RayCasterMut<'w, 's, T = ReadRapierContext<'static, 'static>>
where
	T: SystemParam + 'static,
{
	context: StaticSystemParam<'w, 's, T>,
	colliders: Query<'w, 's, &'static ColliderOf>,
	transforms: Query<'w, 's, (&'static GlobalTransform, Option<&'static AimOffset>)>,
	world_camera: ResMut<'w, WorldCamera>,
}
