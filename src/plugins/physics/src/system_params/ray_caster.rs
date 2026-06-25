mod mouse_hover;
mod mouse_terrain_hover;
mod set_world_camera;
mod solid_objects;
mod terrain;

use crate::components::{collider::ChildColliderOf, offset::AimOffset, world_camera::WorldCamera};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::{self, system_params::MarkUnique, zyheeda_commands::ZyheedaCommands};

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

#[derive(SystemParam)]
pub struct RayCasterMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	world_cameras: Query<'w, 's, (), With<WorldCamera>>,
	_u: MarkUnique<RayCasterMut<'w, 's>>,
}
