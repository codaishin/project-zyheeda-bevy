mod mouse_hover;
mod mouse_terrain_hover;
mod set_world_camera;
mod solid_objects;
mod terrain;

use std::any::type_name;

use crate::components::{collider::ChildColliderOf, offset::AimOffset, world_camera::WorldCamera};
use bevy::{
	ecs::{
		component::ComponentId,
		system::{StaticSystemParam, SystemParam},
	},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::{self, zyheeda_commands::ZyheedaCommands};

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

pub struct RayCasterMut<'w, 's> {
	inner: Inner<'w, 's>,
}

// Manual implementation of `SystemParam` for `RayCasterMut` to prevent multiple access
// of the system parameter within a single system
unsafe impl<'w, 's> SystemParam for RayCasterMut<'w, 's> {
	type State = (<Inner<'w, 's> as SystemParam>::State, ComponentId);
	type Item<'world, 'state> = RayCasterMut<'world, 'state>;

	fn init_state(world: &mut World) -> Self::State {
		(
			Inner::<'w, 's>::init_state(world),
			ResMut::<'w, PreventMultiAccess>::init_state(world),
		)
	}

	fn init_access(
		(state, id): &Self::State,
		system_meta: &mut bevy::ecs::system::SystemMeta,
		component_access_set: &mut bevy::ecs::query::FilteredAccessSet,
		world: &mut World,
	) {
		// Conceptually copied from unique access implementation for resources.
		let combined_access = component_access_set.combined_access();
		if combined_access.has_resource_write(*id) {
			panic!(
				"{} in system {} can only be accessed once",
				type_name::<RayCasterMut>(),
				system_meta.name(),
			);
		}
		component_access_set.add_unfiltered_resource_write(*id);

		Inner::<'w, 's>::init_access(state, system_meta, component_access_set, world);
	}

	unsafe fn get_param<'world, 'state>(
		(state, ..): &'state mut Self::State,
		system_meta: &bevy::ecs::system::SystemMeta,
		world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
		change_tick: bevy::ecs::change_detection::Tick,
	) -> Self::Item<'world, 'state> {
		let inner = unsafe { Inner::<'w, 's>::get_param(state, system_meta, world, change_tick) };

		RayCasterMut { inner }
	}

	fn apply(
		(state, ..): &mut Self::State,
		system_meta: &bevy::ecs::system::SystemMeta,
		world: &mut World,
	) {
		Inner::<'w, 's>::apply(state, system_meta, world);
	}

	fn queue(
		(state, ..): &mut Self::State,
		system_meta: &bevy::ecs::system::SystemMeta,
		world: bevy::ecs::world::DeferredWorld,
	) {
		Inner::<'w, 's>::queue(state, system_meta, world);
	}

	unsafe fn validate_param(
		(state, ..): &mut Self::State,
		system_meta: &bevy::ecs::system::SystemMeta,
		world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell,
	) -> std::prelude::v1::Result<(), bevy::ecs::system::SystemParamValidationError> {
		unsafe { Inner::<'w, 's>::validate_param(state, system_meta, world) }
	}
}

#[derive(SystemParam)]
pub struct Inner<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	world_cameras: Query<'w, 's, (), With<WorldCamera>>,
}

#[derive(Resource)]
struct PreventMultiAccess;
