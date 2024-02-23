use crate::components::{Beam, BeamCommand, BeamConfig, LifeTime};
use bevy::{
	ecs::{
		entity::Entity,
		event::EventReader,
		system::{Commands, Query},
	},
	math::{Ray, Vec3},
	prelude::SpatialBundle,
	transform::{
		components::{GlobalTransform, Transform},
		TransformBundle,
	},
};
use bevy_rapier3d::pipeline::QueryFilter;
use common::{components::GroundOffset, traits::cast_ray::TimeOfImpact};
use interactions::{components::RayCaster, events::RayCastEvent};

pub(crate) fn execute_beam(
	mut commands: Commands,
	mut ray_cast_events: EventReader<RayCastEvent>,
	beam_commands: Query<(Entity, &BeamConfig, &BeamCommand, Option<&Beam>)>,
	transforms: Query<(&GlobalTransform, Option<&GroundOffset>)>,
) {
	for command in &beam_commands {
		ray_cast(&mut commands, &transforms, command);
	}

	for event in ray_cast_events.read() {
		spawn_beam(&mut commands, &beam_commands, event);
		update_beam(&mut commands, &beam_commands, event);
	}
}

fn ray_cast(
	commands: &mut Commands,
	transforms: &Query<(&GlobalTransform, Option<&GroundOffset>)>,
	(id, cfg, cmd, ..): (Entity, &BeamConfig, &BeamCommand, Option<&Beam>),
) {
	let BeamCommand { source, target } = *cmd;
	let Ok((source_transform, source_offset)) = transforms.get(source) else {
		return;
	};
	let Ok((target_transform, target_offset)) = transforms.get(target) else {
		return;
	};
	let Ok(filter) = QueryFilter::default().exclude_rigid_body(source).try_into() else {
		return;
	};
	let Some(mut entity) = commands.get_entity(id) else {
		return;
	};
	let source = translation(source_transform, source_offset);
	let target = translation(target_transform, target_offset);
	entity.try_insert(RayCaster {
		origin: source,
		direction: (target - source).normalize(),
		solid: true,
		filter,
		max_toi: TimeOfImpact(cfg.range),
	});
}

fn spawn_beam(
	commands: &mut Commands,
	beam_commands: &Query<(Entity, &BeamConfig, &BeamCommand, Option<&Beam>)>,
	event: &RayCastEvent,
) {
	let Ok((id, cfg, _, None)) = beam_commands.get(event.source) else {
		return;
	};
	let Some(mut entity) = commands.get_entity(id) else {
		return;
	};

	let (from, to) = get_beam_range(event.target.ray, event.target.toi);
	let transform = get_beam_transform(from, to, event);
	entity.insert((
		SpatialBundle::from_transform(transform),
		Beam {
			from,
			to,
			damage: cfg.damage,
			color: cfg.color,
			emissive: cfg.emissive,
		},
		LifeTime(cfg.lifetime),
	));
}

fn update_beam(
	commands: &mut Commands,
	agents: &Query<(Entity, &BeamConfig, &BeamCommand, Option<&Beam>)>,
	event: &RayCastEvent,
) {
	let Ok((id, _, _, Some(_))) = agents.get(event.source) else {
		return;
	};
	let Some(mut entity) = commands.get_entity(id) else {
		return;
	};

	let (from, to) = get_beam_range(event.target.ray, event.target.toi);
	let transform = get_beam_transform(from, to, event);
	entity.insert(TransformBundle::from(transform));
}

fn translation(transform: &GlobalTransform, offset: Option<&GroundOffset>) -> Vec3 {
	transform.translation() + offset.map_or(Vec3::ZERO, |offset| offset.0)
}

fn get_beam_range(ray: Ray, toi: TimeOfImpact) -> (Vec3, Vec3) {
	(ray.origin, ray.origin + ray.direction * toi.0)
}

fn get_beam_transform(from: Vec3, to: Vec3, event: &RayCastEvent) -> Transform {
	let mut transform = Transform::from_translation((from + to) / 2.).looking_at(to, Vec3::Y);
	transform.scale.z = event.target.toi.0;
	transform
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Beam, BeamConfig, LifeTime};
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		math::{Ray, Vec3},
		prelude::default,
		render::{
			color::Color,
			view::{InheritedVisibility, ViewVisibility, Visibility},
		},
		transform::{components::Transform, TransformBundle},
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
	use std::time::Duration;

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, execute_beam);
		app.add_event::<RayCastEvent>();

		app
	}

	#[test]
	fn insert_ray_caster() {
		let mut app = setup();
		let source = app.world.spawn(GlobalTransform::from_xyz(1., 0., 0.)).id();
		let target = app.world.spawn(GlobalTransform::from_xyz(1., 0., 4.)).id();
		let beam = app
			.world
			.spawn((
				BeamConfig {
					range: 100.,
					..default()
				},
				BeamCommand { source, target },
			))
			.id();

		app.update();

		let ray_caster = app.world.entity(beam).get::<RayCaster>();

		assert_eq!(
			Some(&RayCaster {
				origin: Vec3::new(1., 0., 0.),
				direction: Vec3::new(0., 0., 1.),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			ray_caster
		);
	}

	#[test]
	fn insert_ray_caster_with_ground_offset_for_target() {
		let mut app = setup();
		let source = app.world.spawn(GlobalTransform::from_xyz(1., 0., 0.)).id();
		let target = app
			.world
			.spawn((
				GlobalTransform::from_xyz(1., 0., 4.),
				GroundOffset(Vec3::new(0., 1., 0.)),
			))
			.id();
		let beam = app
			.world
			.spawn((
				BeamConfig {
					range: 100.,
					..default()
				},
				BeamCommand { source, target },
			))
			.id();

		app.update();

		let ray_caster = app.world.entity(beam).get::<RayCaster>();

		assert_eq!(
			Some(&RayCaster {
				origin: Vec3::new(1., 0., 0.),
				direction: Vec3::new(0., 1., 4.).normalize(),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			ray_caster
		);
	}

	#[test]
	fn insert_ray_caster_with_ground_offset_for_source() {
		let mut app = setup();
		let source = app
			.world
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.),
				GroundOffset(Vec3::new(0., 1., 0.)),
			))
			.id();
		let target = app.world.spawn(GlobalTransform::from_xyz(1., 0., 4.)).id();
		let beam = app
			.world
			.spawn((
				BeamConfig {
					range: 100.,
					..default()
				},
				BeamCommand { source, target },
			))
			.id();

		app.update();

		let ray_caster = app.world.entity(beam).get::<RayCaster>();

		assert_eq!(
			Some(&RayCaster {
				origin: Vec3::new(1., 1., 0.),
				direction: Vec3::new(0., -1., 4.).normalize(),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			ray_caster
		);
	}

	#[test]
	fn spawn_beam_from_hit() {
		let mut app = setup();
		let source = app.world.spawn(GlobalTransform::default()).id();
		let beam = app
			.world
			.spawn((
				BeamConfig {
					color: Color::CYAN,
					emissive: Color::ORANGE,
					lifetime: Duration::from_millis(100),
					damage: 42,
					..default()
				},
				BeamCommand {
					source,
					target: Entity::from_raw(default()),
				},
			))
			.id();
		app.world.send_event(RayCastEvent {
			source: beam,
			target: RayCastTarget {
				ray: Ray {
					origin: Vec3::Z,
					direction: Vec3::Y,
				},
				toi: TimeOfImpact(10.),
				..default()
			},
		});

		app.update();

		let beam = app.world.entity(beam);

		assert_eq!(
			(
				Some(&Beam {
					from: Vec3::Z,
					to: Vec3::new(0., 10., 1.),
					damage: 42,
					color: Color::CYAN,
					emissive: Color::ORANGE
				}),
				Some(&LifeTime(Duration::from_millis(100))),
			),
			(beam.get::<Beam>(), beam.get::<LifeTime>())
		);
	}

	#[test]
	fn set_spatial_bundle() {
		let mut app = setup();
		let source = app.world.spawn(GlobalTransform::default()).id();
		let beam = app
			.world
			.spawn((
				BeamConfig::default(),
				BeamCommand {
					source,
					target: Entity::from_raw(default()),
				},
			))
			.id();
		app.world.send_event(RayCastEvent {
			source: beam,
			target: RayCastTarget {
				ray: Ray {
					origin: Vec3::new(0., 1., 0.),
					direction: Vec3::new(1., 0., 0.),
				},
				toi: TimeOfImpact(10.),
				..default()
			},
		});

		app.update();

		let bundle = app
			.world
			.get_entity(beam)
			.and_then(|e| {
				Some(SpatialBundle {
					visibility: *e.get::<Visibility>()?,
					inherited_visibility: *e.get::<InheritedVisibility>()?,
					view_visibility: *e.get::<ViewVisibility>()?,
					transform: *e.get::<Transform>()?,
					global_transform: *e.get::<GlobalTransform>()?,
				})
			})
			.unwrap();

		let mut expected_transform = Transform::from_translation(Vec3::new(5., 1., 0.))
			.looking_at(Vec3::new(10., 1., 0.), Vec3::Y);
		expected_transform.scale.z = 10.;
		let expected_bundle = SpatialBundle::from_transform(expected_transform);

		assert_eq!(
			(
				expected_bundle.visibility,
				expected_bundle.inherited_visibility,
				expected_bundle.view_visibility,
				expected_bundle.transform,
				expected_bundle.global_transform,
			),
			(
				bundle.visibility,
				bundle.inherited_visibility,
				bundle.view_visibility,
				bundle.transform,
				bundle.global_transform,
			)
		);
	}

	#[test]
	fn update_beam() {
		let mut app = setup();
		let source = app.world.spawn(GlobalTransform::default()).id();
		let beam = app
			.world
			.spawn((
				BeamConfig::default(),
				BeamCommand {
					source,
					target: Entity::from_raw(default()),
				},
			))
			.id();
		app.world.send_event(RayCastEvent {
			source: beam,
			target: RayCastTarget {
				ray: Ray {
					origin: Vec3::new(0., 1., 0.),
					direction: Vec3::new(1., 0., 0.),
				},
				toi: TimeOfImpact(10.),
				..default()
			},
		});

		app.update();

		app.update();

		let bundle = app
			.world
			.get_entity(beam)
			.and_then(|e| {
				Some(TransformBundle {
					local: *e.get::<Transform>()?,
					global: *e.get::<GlobalTransform>()?,
				})
			})
			.unwrap();

		let mut expected_transform = Transform::from_translation(Vec3::new(5., 1., 0.))
			.looking_at(Vec3::new(10., 1., 0.), Vec3::Y);
		expected_transform.scale.z = 10.;
		let expected_bundle = TransformBundle::from_transform(expected_transform);

		assert_eq!(
			(expected_bundle.local, expected_bundle.global,),
			(bundle.local, bundle.global,)
		);
	}
}
