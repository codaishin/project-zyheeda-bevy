use crate::{components::RayCaster, events::RayCastEvent};
use bevy::ecs::{
	entity::Entity,
	event::EventWriter,
	system::{Query, Res, Resource},
};
use common::traits::cast_ray::CastRay;

pub(crate) fn execute_ray_caster<TCastRay: CastRay<RayCaster> + Resource>(
	ray_casters: Query<(Entity, &RayCaster)>,
	cast_ray: Res<TCastRay>,
	mut ray_cast_events: EventWriter<RayCastEvent>,
) {
	let hit_a_target = |(source, ray_caster): (Entity, &RayCaster)| {
		cast_ray
			.cast_ray(ray_caster.clone())
			.map(|(target, ..)| (source, target))
	};

	for (source, target) in ray_casters.iter().filter_map(hit_a_target) {
		ray_cast_events.send(RayCastEvent { source, target });
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::RayCastEvent;
	use bevy::{
		app::{App, Update},
		ecs::{entity::Entity, event::Events},
		math::Vec3,
	};
	use bevy_rapier3d::pipeline::QueryFilter;
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
			get_filter: QueryFilter::default,
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
	fn add_cast_ray_event() {
		let mut app = App::new_single_threaded([Update]);
		let mut cast_ray = _CastRay::default();
		cast_ray
			.mock
			.expect_cast_ray()
			.return_const((Entity::from_raw(42), TimeOfImpact::default()));

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
				target: Entity::from_raw(42)
			}],
			events
		);
	}
}
