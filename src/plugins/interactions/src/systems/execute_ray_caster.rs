use crate::{
	components::RayCaster,
	events::{RayCastEvent, RayCastTarget},
};
use bevy::ecs::{
	entity::Entity,
	event::EventWriter,
	query::Added,
	system::{Commands, Query, Res, Resource},
};
use common::traits::cast_ray::CastRay;

pub(crate) fn execute_ray_caster<TCastRay: CastRay<RayCaster> + Resource>(
	mut commands: Commands,
	ray_casters: Query<(Entity, &RayCaster), Added<RayCaster>>,
	cast_ray: Res<TCastRay>,
	mut ray_cast_events: EventWriter<RayCastEvent>,
) {
	for (source, ray_caster) in &ray_casters {
		let hit = cast_ray.cast_ray(ray_caster.clone());
		let max_toi = ray_caster.max_toi;
		let target = match hit {
			None => RayCastTarget::None { max_toi },
			Some((target, toi)) => RayCastTarget::Some { target, toi },
		};
		ray_cast_events.send(RayCastEvent { source, target });
		commands.entity(source).remove::<RayCaster>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::{RayCastEvent, RayCastTarget};
	use bevy::{
		app::{App, Update},
		ecs::{entity::Entity, event::Events},
		math::Vec3,
		utils::default,
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::cast_ray::TimeOfImpact};
	use mockall::{automock, predicate::eq};

	#[derive(Resource, Default)]
	struct _CastRay {
		pub mock: Mock_CastRay,
	}

	#[automock]
	impl CastRay<RayCaster> for _CastRay {
		fn cast_ray(&self, ray: RayCaster) -> Option<(Entity, TimeOfImpact)> {
			self.mock.cast_ray(ray)
		}
	}

	#[test]
	fn cast_ray() {
		let mut app = App::new_single_threaded([Update]);
		let mut cast_ray = _CastRay::default();
		let ray_caster = RayCaster {
			origin: Vec3::ZERO,
			direction: Vec3::ONE,
			max_toi: TimeOfImpact(42.),
			solid: true,
			filter: default(),
		};
		cast_ray
			.mock
			.expect_cast_ray()
			.times(1)
			.with(eq(ray_caster.clone()))
			.return_const(None);

		app.insert_resource(cast_ray);
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, execute_ray_caster::<_CastRay>);
		app.world.spawn(ray_caster);
		app.update();
	}

	#[test]
	fn add_cast_ray_event_with_target() {
		let mut app = App::new_single_threaded([Update]);
		let mut cast_ray = _CastRay::default();
		cast_ray
			.mock
			.expect_cast_ray()
			.return_const((Entity::from_raw(42), TimeOfImpact(42.)));

		app.insert_resource(cast_ray);
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, execute_ray_caster::<_CastRay>);
		let ray_caster = app.world.spawn(RayCaster::default()).id();
		app.update();

		let events = app.world.resource::<Events<RayCastEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events).collect::<Vec<_>>();

		assert_eq!(
			vec![&RayCastEvent {
				source: ray_caster,
				target: RayCastTarget::Some {
					target: Entity::from_raw(42),
					toi: TimeOfImpact(42.)
				}
			}],
			events
		);
	}

	#[test]
	fn add_cast_ray_event_without_target() {
		let mut app = App::new_single_threaded([Update]);
		let mut cast_ray = _CastRay::default();
		cast_ray.mock.expect_cast_ray().return_const(None);

		app.insert_resource(cast_ray);
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, execute_ray_caster::<_CastRay>);
		let ray_caster = app
			.world
			.spawn(RayCaster {
				max_toi: TimeOfImpact(420.),
				..default()
			})
			.id();
		app.update();

		let events = app.world.resource::<Events<RayCastEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events).collect::<Vec<_>>();

		assert_eq!(
			vec![&RayCastEvent {
				source: ray_caster,
				target: RayCastTarget::None {
					max_toi: TimeOfImpact(420.)
				}
			}],
			events
		);
	}

	#[test]
	fn cast_ray_only_once() {
		let mut app = App::new_single_threaded([Update]);
		let mut cast_ray = _CastRay::default();
		let ray_caster = RayCaster {
			origin: Vec3::ZERO,
			direction: Vec3::ONE,
			max_toi: TimeOfImpact(42.),
			solid: true,
			filter: default(),
		};
		cast_ray.mock.expect_cast_ray().times(1).return_const(None);

		app.insert_resource(cast_ray);
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, execute_ray_caster::<_CastRay>);
		app.world.spawn(ray_caster);
		app.update();
		app.update();
	}

	#[test]
	fn remove_ray_caster() {
		let mut app = App::new_single_threaded([Update]);
		let mut cast_ray = _CastRay::default();
		let ray_caster = RayCaster {
			origin: Vec3::ZERO,
			direction: Vec3::ONE,
			max_toi: TimeOfImpact(42.),
			solid: true,
			filter: default(),
		};
		cast_ray.mock.expect_cast_ray().return_const(None);

		app.insert_resource(cast_ray);
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, execute_ray_caster::<_CastRay>);
		let ray_caster = app.world.spawn(ray_caster).id();

		app.update();

		let ray_caster = app.world.entity(ray_caster);

		assert!(!ray_caster.contains::<RayCaster>());
	}
}
