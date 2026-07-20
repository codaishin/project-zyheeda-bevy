use bevy::{
	ecs::{component::ComponentId, system::SystemParam},
	prelude::*,
};
use std::{any::type_name, marker::PhantomData};

/// A system parameter that only tests for uniqueness.
///
/// Can be included in system parameter structs to enforce that it can only be used once
/// within a single system.
///
/// `T` will be used for the displayed message.
pub struct MarkUnique<T>(PhantomData<T>);

unsafe impl<T> SystemParam for MarkUnique<T> {
	type State = ComponentId;
	type Item<'world, 'state> = MarkUnique<T>;

	fn init_state(world: &mut World) -> Self::State {
		ResMut::<'static, PreventMultiAccess>::init_state(world)
	}

	fn init_access(
		state: &Self::State,
		system_meta: &mut bevy::ecs::system::SystemMeta,
		component_access_set: &mut bevy::ecs::query::FilteredAccessSet,
		_: &mut World,
	) {
		// Conceptually copied from unique access implementation for resources.
		let combined_access = component_access_set.combined_access();
		if combined_access.has_write(*state) {
			panic!(
				"{} in system {} can only be accessed once",
				type_name::<T>(),
				system_meta.name(),
			);
		}
		component_access_set.add_resource_write(*state);
	}

	unsafe fn get_param<'world, 'state>(
		_: &'state mut Self::State,
		_: &bevy::ecs::system::SystemMeta,
		_: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
		_: bevy::ecs::change_detection::Tick,
	) -> std::prelude::v1::Result<
		Self::Item<'world, 'state>,
		bevy::ecs::system::SystemParamValidationError,
	> {
		Ok(MarkUnique(PhantomData))
	}
}

#[derive(Resource)]
struct PreventMultiAccess;
