use crate::components::{Beam, BeamCommand, BeamConfig, LifeTime};
use bevy::{
	ecs::{
		entity::Entity,
		event::EventReader,
		query::Added,
		system::{Commands, Query},
	},
	math::{Ray, Vec3},
	transform::components::GlobalTransform,
};
use bevy_rapier3d::pipeline::QueryFilter;
use common::{components::GroundOffset, traits::cast_ray::TimeOfImpact};
use interactions::{
	components::{RayCaster, RayFilter},
	events::{RayCastEvent, RayCastTarget},
};

type CommandComponents<'a> = (
	Entity,
	&'a GlobalTransform,
	Option<&'a GroundOffset>,
	&'a BeamCommand,
);

pub(crate) fn execute_beam(
	mut commands: Commands,
	mut ray_cast_events: EventReader<RayCastEvent>,
	beam_commands: Query<CommandComponents, Added<BeamCommand>>,
	beam_configs: Query<&BeamConfig>,
	targets: Query<(&GlobalTransform, Option<&GroundOffset>)>,
) {
	let origin_and_target = |(id, transform, offset, cmd): CommandComponents| {
		let (target_transform, target_offset) = targets.get(cmd.target).ok()?;
		let target_offset = target_offset.map_or(Vec3::ZERO, |o| o.0);
		let target = target_transform.translation() + target_offset;
		let offset = offset.map_or(Vec3::ZERO, |o| o.0);
		let origin = transform.translation() + offset;
		let filter: RayFilter = QueryFilter::default()
			.exclude_rigid_body(id)
			.try_into()
			.ok()?;
		Some((id, *cmd, origin, target, filter))
	};
	let beam_config = |event: &RayCastEvent| Some((*event, beam_configs.get(event.source).ok()?));

	for (id, cmd, origin, target, filter) in beam_commands.iter().filter_map(origin_and_target) {
		commands.entity(id).insert(RayCaster {
			origin,
			direction: (target - origin).normalize(),
			solid: true,
			filter,
			max_toi: TimeOfImpact(cmd.range),
		});
	}

	for (event, cfg) in ray_cast_events.read().filter_map(beam_config) {
		let (from, to) = match event.target {
			RayCastTarget::Some { ray, toi, .. } => get_beam_range(ray, toi),
			RayCastTarget::None { ray, max_toi } => get_beam_range(ray, max_toi),
		};
		commands.spawn((
			Beam {
				from,
				to,
				color: cfg.color,
				emissive: cfg.emissive,
			},
			LifeTime(cfg.lifetime),
		));
		commands
			.entity(event.source)
			.remove::<(BeamCommand, BeamConfig)>();
	}
}

fn get_beam_range(ray: Ray, toi: TimeOfImpact) -> (Vec3, Vec3) {
	(ray.origin, ray.origin + ray.direction * toi.0)
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use super::*;
	use crate::components::{Beam, BeamConfig, LifeTime};
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		math::{Ray, Vec3},
		prelude::default,
		render::color::Color,
	};
	use bevy_rapier3d::pipeline::QueryFilter;
	use common::{
		components::GroundOffset,
		test_tools::utils::SingleThreadedApp,
		traits::cast_ray::TimeOfImpact,
	};
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
	fn insert_ray_caster() {
		let mut app = setup();
		let target = app.world.spawn(GlobalTransform::from_xyz(1., 0., 4.)).id();
		let beamer = app
			.world
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.),
				BeamConfig::default(),
				BeamCommand {
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
	fn insert_ray_caster_with_ground_offset_for_target() {
		let mut app = setup();
		let target = app
			.world
			.spawn((
				GlobalTransform::from_xyz(1., 0., 4.),
				GroundOffset(Vec3::new(0., 1., 0.)),
			))
			.id();
		let beamer = app
			.world
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.),
				BeamConfig::default(),
				BeamCommand {
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
				direction: Vec3::new(0., 1., 4.).normalize(),
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
	fn insert_ray_caster_with_ground_offset_for_source() {
		let mut app = setup();
		let target = app.world.spawn(GlobalTransform::from_xyz(1., 0., 4.)).id();
		let beamer = app
			.world
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.),
				GroundOffset(Vec3::new(0., 1., 0.)),
				BeamConfig::default(),
				BeamCommand {
					target,
					range: 100.,
				},
			))
			.id();

		app.update();

		let ray_caster = app.world.entity(beamer).get::<RayCaster>();

		assert_eq!(
			Some(&RayCaster {
				origin: Vec3::new(1., 1., 0.),
				direction: Vec3::new(0., -1., 4.).normalize(),
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
	fn do_not_insert_ray_caster_multiple_times() {
		let mut app = setup();
		let target = app.world.spawn(GlobalTransform::from_xyz(1., 0., 4.)).id();
		let beamer = app
			.world
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.),
				BeamConfig::default(),
				BeamCommand {
					target,
					range: 100.,
				},
			))
			.id();

		app.update();

		app.world.entity_mut(beamer).remove::<RayCaster>();

		app.update();

		let ray_caster = app.world.entity(beamer).get::<RayCaster>();

		assert_eq!(None, ray_caster);
	}

	#[test]
	fn spawn_beam_from_hit() {
		let mut app = setup();
		let source = app
			.world
			.spawn((
				GlobalTransform::default(),
				BeamConfig {
					color: Color::CYAN,
					emissive: Color::ORANGE,
					lifetime: Duration::from_millis(100),
				},
				BeamCommand {
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
			.find_map(|e| Some((e.get::<Beam>()?, e.get::<LifeTime>()?)));

		assert_eq!(
			Some((
				&Beam {
					from: Vec3::Z,
					to: Vec3::new(0., 10., 1.),
					color: Color::CYAN,
					emissive: Color::ORANGE
				},
				&LifeTime(Duration::from_millis(100))
			)),
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
				BeamConfig {
					color: Color::CYAN,
					emissive: Color::ORANGE,
					lifetime: Duration::from_millis(1000),
				},
				BeamCommand {
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
			.find_map(|e| Some((e.get::<Beam>()?, e.get::<LifeTime>()?)));

		assert_eq!(
			Some((
				&Beam {
					from: Vec3::Z,
					to: Vec3::new(0., 4., 1.),
					color: Color::CYAN,
					emissive: Color::ORANGE,
				},
				&LifeTime(Duration::from_millis(1000))
			)),
			active_beam
		);
	}

	#[test]
	fn do_not_spawn_when_event_source_not_a_beam_command() {
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

		let active_beam = app.world.iter_entities().find_map(|e| e.get::<Beam>());

		assert_eq!(None, active_beam);
	}

	#[test]
	fn beam_on_not_newly_added_beam_command() {
		let mut app = setup();
		let source = app
			.world
			.spawn((
				GlobalTransform::default(),
				BeamConfig::default(),
				BeamCommand {
					target: Entity::from_raw(default()),
					range: default(),
				},
			))
			.id();

		app.update();

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

		let active_beam = app.world.iter_entities().find_map(|e| e.get::<Beam>());

		assert_eq!(
			Some(&Beam {
				from: Vec3::Z,
				to: Vec3::new(0., 10., 1.),
				..default()
			}),
			active_beam
		);
	}

	#[test]
	fn remove_beam_cmd_when_beam_added() {
		let mut app = setup();
		let source = app
			.world
			.spawn((
				GlobalTransform::default(),
				BeamConfig::default(),
				BeamCommand {
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

		let beam_cmd = app.world.entity(source).get::<BeamCommand>();
		let beam_cfg = app.world.entity(source).get::<BeamConfig>();

		assert_eq!((None, None), (beam_cmd, beam_cfg));
	}
}
