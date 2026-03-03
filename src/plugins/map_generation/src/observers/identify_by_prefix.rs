use bevy::{ecs::system::IntoObserverSystem, gltf::GltfMeshName, prelude::*};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl<T> IdentifyByPrefix for T where T: Component + Default {}

pub(crate) trait IdentifyByPrefix: Component + Default {
	fn identify_by_prefix(prefix: &'static str) -> impl IntoObserverSystem<Add, GltfMeshName, ()> {
		#[rustfmt::skip]
		let observer = move |
			on_add: On<Add, GltfMeshName>,
			mut commands: ZyheedaCommands,
			names: Query<&GltfMeshName>
		| {
			let Ok(GltfMeshName(name)) = names.get(on_add.entity) else {
				return;
			};

			if !name.starts_with(prefix) {
				return;
			}

			commands.try_apply_on(&on_add.entity, |mut e| {
				e.try_insert(Self::default());
			});
		};

		IntoObserverSystem::into_system(observer)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::gltf::GltfMeshName;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Component;

	fn setup(prefix: &'static str) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(_Component::identify_by_prefix(prefix));

		app
	}

	#[test]
	fn insert_mesh_collider() {
		let mut app = setup("Collider");

		let entity = app.world_mut().spawn(GltfMeshName("Collider".to_owned()));

		assert_eq!(Some(&_Component), entity.get::<_Component>());
	}

	#[test_case("Collider", "Foo"; "collider vs foo")]
	#[test_case("Foo", "Collider"; "foo vs collider")]
	fn do_not_insert_mesh_collider_when_not_matching_prefix(
		prefix: &'static str,
		name: &'static str,
	) {
		let mut app = setup(prefix);

		let entity = app.world_mut().spawn(GltfMeshName(name.to_owned()));

		assert_eq!(None, entity.get::<_Component>());
	}

	#[test]
	fn insert_mesh_collider_when_substring_matches() {
		let mut app = setup("Collider");

		let entity = app
			.world_mut()
			.spawn(GltfMeshName("ColliderFooBar".to_owned()));

		assert_eq!(Some(&_Component), entity.get::<_Component>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup("Collider");

		let mut entity = app.world_mut().spawn(GltfMeshName("Collider".to_owned()));
		entity.remove::<_Component>();
		entity.insert(GltfMeshName("Collider".to_owned()));

		assert_eq!(None, entity.get::<_Component>());
	}
}
