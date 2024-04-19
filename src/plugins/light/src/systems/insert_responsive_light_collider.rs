use crate::components::ResponsiveLight;
use bevy::ecs::{
	entity::Entity,
	query::Added,
	system::{Commands, Query},
};
use bevy_rapier3d::geometry::{ActiveEvents, Collider, CollidingEntities, Sensor};
use common::traits::{clamp_zero_positive::ClampZeroPositive, try_insert_on::TryInsertOn};

pub(crate) fn insert_responsive_light_collider(
	new: Query<(Entity, &ResponsiveLight), Added<ResponsiveLight>>,
	mut commands: Commands,
) {
	for (id, light) in &new {
		commands.try_insert_on(
			id,
			(
				Collider::ball(light.data.range.value()),
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
	use crate::components::ResponsiveLightData;
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Handle},
		utils::Uuid,
	};
	use bevy_rapier3d::geometry::ActiveEvents;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::{Intensity, IntensityChangePerSecond, Units},
	};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
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
				model: Entity::from_raw(1),
				light: Entity::from_raw(2),
				data: ResponsiveLightData {
					range: Units::new(42.),
					light_on_material: new_handle(),
					light_off_material: new_handle(),
					max: Intensity::new(100.),
					change: IntensityChangePerSecond::new(10.),
				},
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
				model: Entity::from_raw(1),
				light: Entity::from_raw(2),
				data: ResponsiveLightData {
					range: Units::new(42.),
					light_on_material: new_handle(),
					light_off_material: new_handle(),
					max: Intensity::new(100.),
					change: IntensityChangePerSecond::new(10.),
				},
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
