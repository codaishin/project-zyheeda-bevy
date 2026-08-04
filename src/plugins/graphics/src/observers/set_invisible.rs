use crate::resources::standard_materials::StandardMaterials;
use bevy::{ecs::system::IntoObserverSystem, gltf::GltfMaterialName, prelude::*};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl StandardMaterials {
	pub(crate) fn set_invisible_material(
		invisible: &'static str,
	) -> impl IntoObserverSystem<Add, GltfMaterialName, (), ()> {
		#[rustfmt::skip]
		let system = move |
			on_add: On<Add, GltfMaterialName>,
			mut commands: ZyheedaCommands,
			names: Query<&GltfMaterialName>
		| {
			let Ok(GltfMaterialName(name)) = names.get(on_add.entity) else {
				return;
			};

			if name != invisible {
				return;
			}

			commands.try_apply_on(&on_add.entity, |mut e| {
				e.try_insert(Visibility::Hidden);
			});
		};

		IntoObserverSystem::into_system(system)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup(name: &'static str) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(StandardMaterials::set_invisible_material(name));

		app
	}

	#[test]
	fn set_invisible() {
		let mut app = setup("I hide");

		let entity = app
			.world_mut()
			.spawn(GltfMaterialName(String::from("I hide")));

		assert_eq!(Some(&Visibility::Hidden), entity.get::<Visibility>());
	}

	#[test]
	fn do_not_set_invisible() {
		let mut app = setup("I hide");

		let entity = app
			.world_mut()
			.spawn(GltfMaterialName(String::from("I don't hide")));

		assert_eq!(None, entity.get::<Visibility>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup("I hide");

		let mut entity = app
			.world_mut()
			.spawn(GltfMaterialName(String::from("I hide")));
		entity.remove::<Visibility>();
		entity.insert(GltfMaterialName(String::from("I hide")));

		assert_eq!(None, entity.get::<Visibility>());
	}
}
