mod ground;
mod mouse_ground_hover;
mod mouse_hover;
mod solid_objects;

use crate::components::{no_hover::NoMouseHover, world_camera::WorldCamera};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::components::collider_relationship::ColliderOfInteractionTarget;

#[derive(SystemParam)]
pub struct RayCaster<'w, 's, T = ReadRapierContext<'static, 'static>>
where
	T: SystemParam + 'static,
{
	context: StaticSystemParam<'w, 's, T>,
	interaction_colliders: Query<'w, 's, &'static ColliderOfInteractionTarget>,
	no_mouse_hovers: Query<'w, 's, (), With<NoMouseHover>>,
	world_cams: Query<'w, 's, &'static mut WorldCamera>,
}
