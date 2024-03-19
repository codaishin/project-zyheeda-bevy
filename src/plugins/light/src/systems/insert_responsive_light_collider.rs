use crate::components::ResponsiveLight;
use bevy::ecs::{
	entity::Entity,
	query::Added,
	system::{Commands, Query},
};
use bevy_rapier3d::geometry::{ActiveEvents, Collider, CollidingEntities, Sensor};
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn insert_responsive_light_collider(
	new: Query<(Entity, &ResponsiveLight), Added<ResponsiveLight>>,
	mut commands: Commands,
) {
	for (id, light) in &new {
		commands.try_insert_on(
			id,
			(
				Collider::ball(light.range),
				Sensor,
				ActiveEvents::COLLISION_EVENTS,
				CollidingEntities::default(),
			),
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Handle},
		utils::Uuid,
	};
	use bevy_rapier3d::geometry::ActiveEvents;
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, insert_responsive_light_collider);

		app
	}

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	#[test]
	fn add_sensor_collider() {
		let mut app = setup();
		let light = app
			.world
			.spawn(ResponsiveLight {
				range: 42.,
				model: Entity::from_raw(1),
				light: Entity::from_raw(2),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
			})
			.id();

		app.update();

		let light = app.world.entity(light);

		assert_eq!(
			(
				true,
				true,
				Some(Collider::ball(42.).raw.as_ball()),
				Some(&ActiveEvents::COLLISION_EVENTS)
			),
			(
				light.contains::<Sensor>(),
				light.contains::<CollidingEntities>(),
				light.get::<Collider>().map(|c| c.raw.as_ball()),
				light.get::<ActiveEvents>()
			)
		)
	}

	#[test]
	fn add_only_when_new() {
		let mut app = setup();
		let light = app
			.world
			.spawn(ResponsiveLight {
				range: 42.,
				model: Entity::from_raw(1),
				light: Entity::from_raw(2),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
			})
			.id();

		app.update();

		app.world
			.entity_mut(light)
			.remove::<(Sensor, Collider, ActiveEvents, CollidingEntities)>();

		app.update();

		let light = app.world.entity(light);

		assert_eq!(
			(false, false, None, None),
			(
				light.contains::<Sensor>(),
				light.contains::<CollidingEntities>(),
				light.get::<Collider>().map(|c| c.raw.as_ball()),
				light.get::<ActiveEvents>()
			)
		)
	}
}
