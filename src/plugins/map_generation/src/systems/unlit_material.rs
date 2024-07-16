use crate::components::Unlit;
use bevy::{
	asset::{Assets, Handle},
	ecs::{
		query::Added,
		system::{Query, ResMut},
	},
	pbr::StandardMaterial,
};

pub(crate) fn unlit_material(
	handles: Query<&Handle<StandardMaterial>, Added<Unlit>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	for handle in &handles {
		let Some(material) = materials.get_mut(handle) else {
			continue;
		};
		material.unlit = true;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::Assets,
		pbr::StandardMaterial,
		utils::default,
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Assets<StandardMaterial>>();
		app.add_systems(Update, unlit_material);

		app
	}

	#[test]
	fn set_material_unlit() {
		let mut app = setup();
		let material = app
			.world_mut()
			.resource_mut::<Assets<StandardMaterial>>()
			.add(StandardMaterial {
				unlit: false,
				..default()
			});
		app.world_mut().spawn((material.clone(), Unlit));

		app.update();

		let material = app
			.world()
			.resource::<Assets<StandardMaterial>>()
			.get(&material);

		assert_eq!(Some(true), material.map(|m| m.unlit));
	}

	#[test]
	fn do_not_set_material_unlit_when_not_new() {
		let mut app = setup();
		let material = app
			.world_mut()
			.resource_mut::<Assets<StandardMaterial>>()
			.add(StandardMaterial {
				unlit: false,
				..default()
			});
		app.world_mut().spawn((material.clone(), Unlit));

		app.update();

		app.world_mut()
			.resource_mut::<Assets<StandardMaterial>>()
			.get_mut(&material)
			.unwrap()
			.unlit = false;

		app.update();

		let material = app
			.world()
			.resource::<Assets<StandardMaterial>>()
			.get(&material);

		assert_eq!(Some(false), material.map(|m| m.unlit));
	}
}
