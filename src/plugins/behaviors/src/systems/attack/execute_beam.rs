use crate::components::{Beam, BeamCommand, BeamConfig, LifeTime};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		event::EventReader,
		removal_detection::RemovedComponents,
		system::{Commands, Query},
	},
	hierarchy::DespawnRecursiveExt,
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

type BeamCommandComponents<'a> = (
	Entity,
	&'a GlobalTransform,
	Option<&'a GroundOffset>,
	&'a BeamConfig,
	&'a BeamCommand,
	Option<&'a SustainsBeam>,
);

#[derive(Component, Debug, PartialEq)]
pub(crate) struct SustainsBeam(Entity);

pub(crate) fn execute_beam(
	mut commands: Commands,
	mut ray_cast_events: EventReader<RayCastEvent>,
	agents: Query<BeamCommandComponents>,
	sustains: Query<&SustainsBeam>,
	beams: Query<&Beam>,
	targets: Query<(&GlobalTransform, Option<&GroundOffset>)>,
	mut removed_commands: RemovedComponents<BeamCommand>,
) {
	for agent in &agents {
		ray_cast(&mut commands, &targets, agent);
	}

	for event in ray_cast_events.read() {
		spawn_beam(&mut commands, &agents, event);
		update_beam(&mut commands, &agents, event, &beams);
	}

	for id in removed_commands.read() {
		remove_outdated_beams(&mut commands, &sustains, id);
	}
}

fn ray_cast(
	commands: &mut Commands,
	targets: &Query<(&GlobalTransform, Option<&GroundOffset>)>,
	(id, origin_transform, origin_offset, cfg, cmd, ..): BeamCommandComponents,
) {
	let Ok((target_transform, target_offset)) = targets.get(cmd.target) else {
		return;
	};
	let Ok(filter) = QueryFilter::default().exclude_rigid_body(id).try_into() else {
		return;
	};
	let origin = translation(origin_transform, origin_offset);
	let target = translation(target_transform, target_offset);
	commands.entity(id).insert(RayCaster {
		origin,
		direction: (target - origin).normalize(),
		solid: true,
		filter,
		max_toi: TimeOfImpact(cfg.range),
	});
}

fn spawn_beam(
	commands: &mut Commands,
	agents: &Query<BeamCommandComponents>,
	event: &RayCastEvent,
) {
	let Ok((_, _, _, cfg, _, None)) = agents.get(event.source) else {
		return;
	};

	let (from, to) = get_beam_range(event.target.ray, event.target.toi);
	let transform = get_beam_transform(from, to, event);
	let beam = commands
		.spawn((
			SpatialBundle::from_transform(transform),
			Beam {
				from,
				to,
				damage: cfg.damage,
				color: cfg.color,
				emissive: cfg.emissive,
			},
			LifeTime(cfg.lifetime),
		))
		.id();
	commands.entity(event.source).insert(SustainsBeam(beam));
}

fn update_beam(
	commands: &mut Commands,
	agents: &Query<BeamCommandComponents>,
	event: &RayCastEvent,
	beams: &Query<&Beam>,
) {
	let Ok((.., Some(sustains))) = agents.get(event.source) else {
		return;
	};

	if !beams.contains(sustains.0) {
		commands
			.entity(event.source)
			.remove::<(BeamCommand, BeamConfig, SustainsBeam)>();
		return;
	}

	let (from, to) = get_beam_range(event.target.ray, event.target.toi);
	let transform = get_beam_transform(from, to, event);
	commands
		.entity(sustains.0)
		.insert(TransformBundle::from(transform));
}

fn remove_outdated_beams(commands: &mut Commands, sustains: &Query<&SustainsBeam>, id: Entity) {
	if let Ok(sustain) = sustains.get(id) {
		commands.entity(sustain.0).despawn_recursive();
	};
	commands.entity(id).remove::<SustainsBeam>();
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
		hierarchy::BuildWorldChildren,
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
		let target = app.world.spawn(GlobalTransform::from_xyz(1., 0., 4.)).id();
		let beamer = app
			.world
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.),
				BeamConfig {
					range: 100.,
					..default()
				},
				BeamCommand { target },
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
				BeamConfig {
					range: 100.,
					..default()
				},
				BeamCommand { target },
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
				BeamConfig {
					range: 100.,
					..default()
				},
				BeamCommand { target },
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
					damage: 42,
					..default()
				},
				BeamCommand {
					target: Entity::from_raw(default()),
				},
			))
			.id();
		app.world.send_event(RayCastEvent {
			source,
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

		let (beam_id, beam, beam_lifetime) = app
			.world
			.iter_entities()
			.find_map(|e| Some((e.id(), e.get::<Beam>()?, e.get::<LifeTime>()?)))
			.unwrap();
		let sustains_beam = app.world.entity(source).get::<SustainsBeam>().unwrap();

		assert_eq!(
			(
				&Beam {
					from: Vec3::Z,
					to: Vec3::new(0., 10., 1.),
					damage: 42,
					color: Color::CYAN,
					emissive: Color::ORANGE
				},
				&LifeTime(Duration::from_millis(100)),
				&SustainsBeam(beam_id)
			),
			(beam, beam_lifetime, sustains_beam)
		);
	}

	#[test]
	fn do_not_spawn_when_event_source_sustains_a_beam_already() {
		let mut app = setup();
		let fake_beam = app.world.spawn_empty().id();
		let source = app
			.world
			.spawn((
				SustainsBeam(fake_beam),
				GlobalTransform::default(),
				BeamConfig::default(),
				BeamCommand {
					target: Entity::from_raw(default()),
				},
			))
			.id();
		app.world.send_event(RayCastEvent {
			source,
			target: RayCastTarget {
				ray: Ray {
					origin: Vec3::Z,
					direction: Vec3::Y,
				},
				toi: TimeOfImpact(4.),
				..default()
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
				},
			))
			.id();

		app.update();

		app.world.send_event(RayCastEvent {
			source,
			target: RayCastTarget {
				entity: Some(Entity::from_raw(default())),
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
	fn set_spatial_bundle() {
		let mut app = setup();
		let source = app
			.world
			.spawn((
				GlobalTransform::default(),
				BeamConfig::default(),
				BeamCommand {
					target: Entity::from_raw(default()),
				},
			))
			.id();
		app.world.send_event(RayCastEvent {
			source,
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
			.iter_entities()
			.filter(|e| e.contains::<Beam>())
			.find_map(|e| {
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
	fn update_sustained_beam() {
		let mut app = setup();
		let beam = app.world.spawn(Beam::default()).id();
		let source = app
			.world
			.spawn((
				SustainsBeam(beam),
				GlobalTransform::default(),
				BeamConfig::default(),
				BeamCommand {
					target: Entity::from_raw(default()),
				},
			))
			.id();
		app.world.send_event(RayCastEvent {
			source,
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
			.iter_entities()
			.filter(|e| e.contains::<Beam>())
			.find_map(|e| {
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

	#[test]
	fn remove_beam_control_components() {
		let mut app = setup();
		let non_beam = app.world.spawn_empty().id();
		let source = app
			.world
			.spawn((
				SustainsBeam(non_beam),
				GlobalTransform::default(),
				BeamConfig::default(),
				BeamCommand {
					target: Entity::from_raw(default()),
				},
			))
			.id();
		app.world.send_event(RayCastEvent {
			source,
			target: RayCastTarget::default(),
		});

		app.update();

		let source = app.world.entity(source);

		assert_eq!(
			(None, None, None),
			(
				source.get::<SustainsBeam>(),
				source.get::<BeamConfig>(),
				source.get::<BeamCommand>(),
			)
		);
	}

	#[test]
	fn remove_beam_and_sustain_when_beam_command_gone() {
		let mut app = setup();
		let beam = app.world.spawn(Beam::default()).id();
		let child = app.world.spawn_empty().set_parent(beam).id();
		let source = app
			.world
			.spawn((
				SustainsBeam(beam),
				GlobalTransform::default(),
				BeamConfig::default(),
				BeamCommand {
					target: Entity::from_raw(default()),
				},
			))
			.id();

		app.update();

		app.world.entity_mut(source).remove::<BeamCommand>();

		app.update();

		let beams = app.world.iter_entities().filter_map(|e| e.get::<Beam>());
		let sustains = app
			.world
			.iter_entities()
			.filter_map(|e| e.get::<SustainsBeam>());

		assert_eq!(
			(0, 0, true),
			(
				beams.count(),
				sustains.count(),
				app.world.get_entity(child).is_none()
			)
		);
	}
}
