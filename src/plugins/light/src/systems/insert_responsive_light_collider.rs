use crate::components::responsive_light::ResponsiveLight;
use bevy::prelude::*;
use bevy_rapier3d::geometry::{ActiveEvents, Collider, CollidingEntities, Sensor};
use common::traits::try_insert_on::TryInsertOn;
use std::ops::Deref;

pub(crate) fn insert_responsive_light_collider(
	new: Query<(Entity, &ResponsiveLight), Added<ResponsiveLight>>,
	mut commands: Commands,
) {
	for (id, light) in &new {
		commands.try_insert_on(
			id,
			(
				TransformBundle::default(),
				Collider::ball(*light.range.deref()),
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
	use bevy_rapier3d::geometry::ActiveEvents;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::{Intensity, IntensityChangePerSecond, Units},
		traits::clamp_zero_positive::ClampZeroPositive,
	};
	use uuid::Uuid;

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
			.world_mut()
			.spawn(ResponsiveLight {
				model: Entity::from_raw(1),
				light: Entity::from_raw(2),
				range: Units::new(42.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(10.),
			})
			.id();

		app.update();

		let light = app.world().entity(light);

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
			.world_mut()
			.spawn(ResponsiveLight {
				model: Entity::from_raw(1),
				light: Entity::from_raw(2),
				range: Units::new(42.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(10.),
			})
			.id();

		app.update();

		app.world_mut()
			.entity_mut(light)
			.remove::<(Sensor, Collider, ActiveEvents, CollidingEntities)>();

		app.update();

		let light = app.world().entity(light);

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
