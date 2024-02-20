use crate::components::{ActiveBeam, Beam};
use bevy::{
	ecs::{
		entity::Entity,
		event::EventReader,
		system::{Commands, Query},
	},
	math::{Ray, Vec3},
	render::color::Color,
	transform::components::GlobalTransform,
};
use bevy_rapier3d::pipeline::QueryFilter;
use common::traits::cast_ray::TimeOfImpact;
use interactions::{
	components::{RayCaster, RayFilter},
	events::{RayCastEvent, RayCastTarget},
};

pub(crate) fn execute_beam(
	mut commands: Commands,
	mut ray_cast_events: EventReader<RayCastEvent>,
	beamers: Query<(Entity, &GlobalTransform, &Beam)>,
	transforms: Query<&GlobalTransform>,
) {
	let origin_and_target = |(id, transform, beam): (Entity, &GlobalTransform, &Beam)| {
		let target = transforms.get(beam.target).ok()?.translation();
		let filter: RayFilter = QueryFilter::default()
			.exclude_rigid_body(id)
			.try_into()
			.ok()?;
		Some((id, *beam, transform.translation(), target, filter))
	};
	let event_source_is_beam = |event: &&RayCastEvent| beamers.contains(event.source);

	for (id, beam, origin, target, filter) in beamers.iter().filter_map(origin_and_target) {
		commands.entity(id).insert(RayCaster {
			origin,
			direction: (target - origin).normalize(),
			solid: true,
			filter,
			max_toi: TimeOfImpact(beam.range),
		});
	}

	for event in ray_cast_events.read().filter(event_source_is_beam) {
		let (from, to) = match event.target {
			RayCastTarget::Some { ray, toi, .. } => get_beam_range(ray, toi),
			RayCastTarget::None { ray, max_toi } => get_beam_range(ray, max_toi),
		};
		commands.spawn(ActiveBeam {
			from,
			to,
			color: Color::BLACK,
			emission: Color::rgb_linear(13.99, 13.99, 13.99),
		});
	}
}

fn get_beam_range(ray: Ray, toi: TimeOfImpact) -> (Vec3, Vec3) {
	(ray.origin, ray.origin + ray.direction * toi.0)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::ActiveBeam;
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		math::{Ray, Vec3},
		prelude::default,
		render::color::Color,
	};
	use bevy_rapier3d::pipeline::QueryFilter;
	use common::{test_tools::utils::SingleThreadedApp, traits::cast_ray::TimeOfImpact};
	use interactions::{
		components::RayCaster,
		events::{RayCastEvent, RayCastTarget},
	};

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, execute_beam);
		app.add_event::<RayCastEvent>();

		app
	}

	#[test]
	fn spawn_ray_caster() {
		let mut app = setup();
		let target = app.world.spawn(GlobalTransform::from_xyz(1., 0., 4.)).id();
		let beamer = app
			.world
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.),
				Beam {
					target,
					range: 100.,
				},
			))
			.id();

		app.update();

		let ray_caster = app.world.entity(beamer).get::<RayCaster>();

		assert_eq!(
			Some(&RayCaster {
				origin: Vec3::new(1., 0., 0.),
				direction: Vec3::new(0., 0., 1.),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(beamer)
					.try_into()
					.unwrap(),
			}),
			ray_caster
		);
	}

	#[test]
	fn spawn_beam_from_hit() {
		let mut app = setup();
		let source = app
			.world
			.spawn((
				GlobalTransform::default(),
				Beam {
					target: Entity::from_raw(default()),
					range: default(),
				},
			))
			.id();
		app.world.send_event(RayCastEvent {
			source,
			target: RayCastTarget::Some {
				target: Entity::from_raw(default()),
				ray: Ray {
					origin: Vec3::Z,
					direction: Vec3::Y,
				},
				toi: TimeOfImpact(10.),
			},
		});

		app.update();

		let active_beam = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<ActiveBeam>());

		assert_eq!(
			Some(&ActiveBeam {
				from: Vec3::Z,
				to: Vec3::new(0., 10., 1.),
				// FIXME: Color values need to be configurable in some way
				color: Color::BLACK,
				emission: Color::rgb_linear(13.99, 13.99, 13.99)
			}),
			active_beam
		);
	}

	#[test]
	fn spawn_beam_from_miss() {
		let mut app = setup();
		let source = app
			.world
			.spawn((
				GlobalTransform::default(),
				Beam {
					target: Entity::from_raw(default()),
					range: default(),
				},
			))
			.id();
		app.world.send_event(RayCastEvent {
			source,
			target: RayCastTarget::None {
				ray: Ray {
					origin: Vec3::Z,
					direction: Vec3::Y,
				},
				max_toi: TimeOfImpact(4.),
			},
		});

		app.update();

		let active_beam = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<ActiveBeam>());

		assert_eq!(
			Some(&ActiveBeam {
				from: Vec3::Z,
				to: Vec3::new(0., 4., 1.),
				color: Color::BLACK,
				emission: Color::rgb_linear(13.99, 13.99, 13.99)
			}),
			active_beam
		);
	}

	#[test]
	fn do_not_spawn_when_event_source_not_a_beam() {
		let mut app = setup();
		let source = app.world.spawn_empty().id();
		app.world.send_event(RayCastEvent {
			source,
			target: RayCastTarget::Some {
				target: Entity::from_raw(42),
				ray: Ray {
					origin: Vec3::Z,
					direction: Vec3::Y,
				},
				toi: TimeOfImpact(4.),
			},
		});

		app.update();

		let active_beam = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<ActiveBeam>());

		assert_eq!(None, active_beam);
	}
}
