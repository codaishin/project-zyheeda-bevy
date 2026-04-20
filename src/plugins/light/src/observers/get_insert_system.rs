use bevy::{ecs::system::IntoObserverSystem, prelude::*};
use common::{
	components::asset_mesh_name::AssetMeshName,
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};
use zyheeda_core::prelude::*;

impl<T> GetInsertObserver for T where T: Component + Default + TargetMeshName {}

pub(crate) trait GetInsertObserver: Component + Default + TargetMeshName {
	fn get_insert_observer() -> impl IntoObserverSystem<Add, AssetMeshName, ()> {
		let target_mesh_name = Self::target_mesh_name();

		#[rustfmt::skip]
		let observer = move |
			added_name: On<Add, AssetMeshName>,
			lights: Query<&AssetMeshName>,
			mut commands: ZyheedaCommands,
		| {
			let Ok(AssetMeshName(mesh_name)) = lights.get(added_name.entity) else {
				return;
			};

			if mesh_name != &target_mesh_name {
				return;
			}

			commands.try_apply_on(&added_name.entity, |mut e| {
				e.try_insert(Self::default());
			});
		};

		IntoObserverSystem::into_system(observer)
	}
}

pub(crate) trait TargetMeshName {
	fn target_mesh_name() -> NormalizedNameLazy;
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::components::asset_mesh_name::AssetMeshName;
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Component;

	impl TargetMeshName for _Component {
		fn target_mesh_name() -> NormalizedNameLazy {
			NormalizedNameLazy::from_name("TargetName")
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(_Component::get_insert_observer());

		app
	}

	#[test]
	fn insert_component() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(AssetMeshName::normalized("TargetName"));

		assert_eq!(Some(&_Component), entity.get::<_Component>());
	}

	#[test]
	fn do_not_insert_when_name_not_valid() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(AssetMeshName::normalized("NotTargetName"));

		assert_eq!(None, entity.get::<_Component>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();

		let mut entity = app
			.world_mut()
			.spawn(AssetMeshName::normalized("TargetName"));
		entity.remove::<_Component>();
		entity.insert(AssetMeshName::normalized("TargetName"));

		assert_eq!(None, entity.get::<_Component>());
	}
}
