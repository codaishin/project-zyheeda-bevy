use crate::{
	components::RayCaster,
	events::{RayCastEvent, RayCastInfo},
};
use bevy::{
	ecs::{
		entity::Entity,
		event::EventWriter,
		query::Added,
		system::{Commands, Query, Res, Resource},
	},
	math::Ray3d,
};
use common::traits::{cast_ray::CastRayContinuously, try_remove_from::TryRemoveFrom};

pub(crate) fn execute_ray_caster<TCastRay: CastRayContinuously<RayCaster> + Resource>(
	mut commands: Commands,
	ray_casters: Query<(Entity, &RayCaster), Added<RayCaster>>,
	cast_ray: Res<TCastRay>,
	mut ray_cast_events: EventWriter<RayCastEvent>,
) {
	for (source, ray_caster) in &ray_casters {
		let info = RayCastInfo {
			hits: cast_ray.cast_ray_continuously(ray_caster),
			ray: Ray3d {
				origin: ray_caster.origin,
				direction: ray_caster.direction,
			},
			max_toi: ray_caster.max_toi,
		};
		ray_cast_events.send(RayCastEvent { source, info });
		commands.try_remove_from::<RayCaster>(source);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::{RayCastEvent, RayCastInfo};
	use bevy::{
		app::{App, Update},
		ecs::{entity::Entity, event::Events},
		math::{Dir3, Vec3},
		utils::default,
	};
	use bevy_rapier3d::math::Real;
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{cast_ray::TimeOfImpact, nested_mock::NestedMock},
	};
	use macros::NestedMock;
	use mockall::{automock, predicate::eq};

	#[derive(Resource, NestedMock)]
	struct _CastRay {
		pub mock: Mock_CastRay,
	}

	#[automock]
	impl CastRayContinuously<RayCaster> for _CastRay {
		fn cast_ray_continuously(&self, ray: &RayCaster) -> Vec<(Entity, TimeOfImpact)> {
			self.mock.cast_ray_continuously(ray)
		}
	}

	#[test]
	fn cast_ray() {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(_CastRay::new_mock(|mock| {
			mock.expect_cast_ray_continuously()
				.times(1)
				.with(eq(RayCaster {
					origin: Vec3::ZERO,
					direction: Dir3::NEG_Y,
					max_toi: TimeOfImpact(42.),
					solid: true,
					filter: default(),
				}))
				.return_const(vec![]);
		}));
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, execute_ray_caster::<_CastRay>);
		app.world_mut().spawn(RayCaster {
			origin: Vec3::ZERO,
			direction: Dir3::NEG_Y,
			max_toi: TimeOfImpact(42.),
			solid: true,
			filter: default(),
		});

		app.update();
	}

	#[test]
	fn add_cast_ray_event_with_target() {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(_CastRay::new_mock(|mock| {
			mock.expect_cast_ray_continuously()
				.return_const(vec![(Entity::from_raw(42), TimeOfImpact(42.))]);
		}));
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, execute_ray_caster::<_CastRay>);
		let ray_caster = app
			.world_mut()
			.spawn(RayCaster {
				origin: Vec3::ONE,
				direction: Dir3::Y,
				max_toi: TimeOfImpact(Real::INFINITY),
				..default()
			})
			.id();

		app.update();

		let events = app.world().resource::<Events<RayCastEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events).collect::<Vec<_>>();

		assert_eq!(
			vec![&RayCastEvent {
				source: ray_caster,
				info: RayCastInfo {
					hits: vec![(Entity::from_raw(42), TimeOfImpact(42.))],
					ray: Ray3d {
						origin: Vec3::ONE,
						direction: Dir3::Y
					},
					max_toi: TimeOfImpact(Real::INFINITY)
				}
			}],
			events
		);
	}

	#[test]
	fn add_cast_ray_event_without_target() {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(_CastRay::new_mock(|mock| {
			mock.expect_cast_ray_continuously().return_const(vec![]);
		}));
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, execute_ray_caster::<_CastRay>);
		let ray_caster = app
			.world_mut()
			.spawn(RayCaster {
				max_toi: TimeOfImpact(420.),
				origin: Vec3::ONE,
				direction: Dir3::Y,
				..default()
			})
			.id();

		app.update();

		let events = app.world().resource::<Events<RayCastEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events).collect::<Vec<_>>();

		assert_eq!(
			vec![&RayCastEvent {
				source: ray_caster,
				info: RayCastInfo {
					hits: vec![],
					ray: Ray3d {
						origin: Vec3::ONE,
						direction: Dir3::Y
					},
					max_toi: TimeOfImpact(420.)
				}
			}],
			events
		);
	}

	#[test]
	fn cast_ray_only_once() {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(_CastRay::new_mock(|mock| {
			mock.expect_cast_ray_continuously()
				.times(1)
				.return_const(vec![]);
		}));
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, execute_ray_caster::<_CastRay>);
		app.world_mut().spawn(RayCaster {
			origin: Vec3::ZERO,
			direction: Dir3::NEG_Y,
			max_toi: TimeOfImpact(42.),
			solid: true,
			filter: default(),
		});

		app.update();
		app.update();
	}

	#[test]
	fn remove_ray_caster() {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(_CastRay::new_mock(|mock| {
			mock.expect_cast_ray_continuously().return_const(vec![]);
		}));
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, execute_ray_caster::<_CastRay>);
		let ray_caster = app
			.world_mut()
			.spawn(RayCaster {
				origin: Vec3::ZERO,
				direction: Dir3::NEG_Y,
				max_toi: TimeOfImpact(42.),
				solid: true,
				filter: default(),
			})
			.id();

		app.update();

		let ray_caster = app.world().entity(ray_caster);

		assert!(!ray_caster.contains::<RayCaster>());
	}
}
