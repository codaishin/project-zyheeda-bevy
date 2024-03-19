use crate::components::{ResponsiveLight, ResponsiveLightTrigger};
use bevy::{
	asset::Handle,
	ecs::{
		component::Component,
		query::Changed,
		system::{Commands, Query},
	},
	pbr::StandardMaterial,
	render::view::Visibility,
};
use common::traits::{has_collisions::HasCollisions, try_insert_on::TryInsertOn};

pub(crate) fn responsive_light<TColliderCollection: HasCollisions + Component>(
	mut commands: Commands,
	responsive_lights: Query<
		(&ResponsiveLight, &TColliderCollection),
		Changed<TColliderCollection>,
	>,
	triggers: Query<&ResponsiveLightTrigger>,
) {
	for (responsive, collisions) in &responsive_lights {
		let (visibility, material) = components(responsive, collisions, &triggers);
		commands.try_insert_on(responsive.light, visibility);
		commands.try_insert_on(responsive.model, material);
	}
}

fn components<TColliderCollection: HasCollisions>(
	responsive: &ResponsiveLight,
	collisions: &TColliderCollection,
	triggers: &Query<&ResponsiveLightTrigger>,
) -> (Visibility, Handle<StandardMaterial>) {
	if collisions.collisions().any(|e| triggers.contains(e)) {
		return (Visibility::Visible, responsive.light_on_material.clone());
	}

	(Visibility::Hidden, responsive.light_off_material.clone())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::ResponsiveLightTrigger;
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Handle},
		ecs::entity::Entity,
		pbr::StandardMaterial,
		render::view::Visibility,
		utils::Uuid,
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Collisions(Vec<Entity>);

	impl HasCollisions for _Collisions {
		fn collisions(&self) -> impl Iterator<Item = Entity> + '_ {
			self.0.iter().cloned()
		}
	}

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, responsive_light::<_Collisions>);

		app
	}

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	#[test]
	fn apply_on() {
		let mut app = setup();
		let light_on_material = new_handle();
		let trigger = app.world.spawn(ResponsiveLightTrigger).id();
		let model = app.world.spawn_empty().id();
		let light = app.world.spawn_empty().id();

		app.world.spawn((
			ResponsiveLight {
				model,
				light,
				range: 0.,
				light_on_material: light_on_material.clone(),
				light_off_material: new_handle(),
			},
			_Collisions(vec![trigger]),
		));

		app.update();

		assert_eq!(
			(Some(&Visibility::Visible), Some(&light_on_material)),
			(
				app.world.entity(light).get::<Visibility>(),
				app.world.entity(model).get::<Handle<StandardMaterial>>()
			)
		)
	}

	#[test]
	fn apply_off() {
		let mut app = setup();
		let light_off_material = new_handle();
		let model = app.world.spawn_empty().id();
		let light = app.world.spawn_empty().id();

		app.world.spawn((
			ResponsiveLight {
				model,
				light,
				range: 0.,
				light_off_material: light_off_material.clone(),
				light_on_material: new_handle(),
			},
			_Collisions(vec![]),
		));

		app.update();

		assert_eq!(
			(Some(&Visibility::Hidden), Some(&light_off_material)),
			(
				app.world.entity(light).get::<Visibility>(),
				app.world.entity(model).get::<Handle<StandardMaterial>>()
			)
		)
	}
}
