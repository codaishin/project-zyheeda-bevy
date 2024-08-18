use crate::components::{Beam, BeamCommand, BeamConfig, LifeTime};
use bevy::{
	ecs::{
		entity::Entity,
		event::EventReader,
		system::{Commands, Query},
	},
	hierarchy::DespawnRecursiveExt,
	math::{Dir3, Vec3},
	prelude::SpatialBundle,
	transform::{
		bundles::TransformBundle,
		components::{GlobalTransform, Transform},
	},
};
use bevy_rapier3d::pipeline::{QueryFilter, QueryFilterFlags};
use common::{components::GroundOffset, traits::cast_ray::TimeOfImpact};
use interactions::{
	components::{RayCaster, RayFilter},
	events::{InteractionEvent, Ray},
};

pub(crate) fn execute_beam(
	mut commands: Commands,
	mut ray_cast_events: EventReader<InteractionEvent<Ray>>,
	beam_commands: Query<(Entity, &BeamConfig, &BeamCommand, Option<&Beam>)>,
	transforms: Query<(&GlobalTransform, Option<&GroundOffset>)>,
) {
	for beam_command in &beam_commands {
		ray_cast_or_despawn(&mut commands, &transforms, beam_command);
	}

	for event in ray_cast_events.read() {
		spawn_beam(&mut commands, &beam_commands, event);
		update_beam(&mut commands, &beam_commands, event);
	}
}

fn ray_cast_or_despawn(
	commands: &mut Commands,
	transforms: &Query<(&GlobalTransform, Option<&GroundOffset>)>,
	beam_command: (Entity, &BeamConfig, &BeamCommand, Option<&Beam>),
) {
	let (id, _, BeamCommand { source, .. }, ..) = beam_command;
	let source_exists = commands.get_entity(*source).is_some();
	match (source_exists, commands.get_entity(id)) {
		(false, Some(beam)) => beam.despawn_recursive(),
		(true, _) => ray_cast(commands, transforms, beam_command),
		_ => {}
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
	let Some(filter) = get_filter(source) else {
		return;
	};
	let Some(mut beam) = commands.get_entity(id) else {
		return;
	};
	let origin = translation(source_transform, source_offset);
	let target = translation(target_transform, target_offset);
	let Ok(direction) = Dir3::new(target - origin) else {
		return;
	};
	beam.try_insert(RayCaster {
		origin,
		direction,
		solid: true,
		filter,
		max_toi: TimeOfImpact(cfg.range),
	});
}

fn get_filter(source: Entity) -> Option<RayFilter> {
	QueryFilter::from(QueryFilterFlags::EXCLUDE_SENSORS)
		.exclude_rigid_body(source)
		.try_into()
		.ok()
}

fn spawn_beam(
	commands: &mut Commands,
	beam_commands: &Query<(Entity, &BeamConfig, &BeamCommand, Option<&Beam>)>,
	InteractionEvent(source, ray): &InteractionEvent<Ray>,
) {
	let Ok((id, cfg, _, None)) = beam_commands.get(*source) else {
		return;
	};
	let Some(mut beam) = commands.get_entity(id) else {
		return;
	};

	let (from, to) = get_beam_range(ray);
	let transform = get_beam_transform(from, to, ray);
	beam.try_insert((
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
	InteractionEvent(source, ray): &InteractionEvent<Ray>,
) {
	let Ok((id, _, _, Some(_))) = agents.get(*source) else {
		return;
	};
	let Some(mut beam) = commands.get_entity(id) else {
		return;
	};

	let (from, to) = get_beam_range(ray);
	let transform = get_beam_transform(from, to, ray);
	beam.try_insert(TransformBundle::from(transform));
}

fn translation(transform: &GlobalTransform, offset: Option<&GroundOffset>) -> Vec3 {
	transform.translation() + offset.map_or(Vec3::ZERO, |offset| offset.0)
}

fn get_beam_range(Ray(ray, TimeOfImpact(toi)): &Ray) -> (Vec3, Vec3) {
	(ray.origin, ray.origin + *ray.direction * *toi)
}

fn get_beam_transform(from: Vec3, to: Vec3, Ray(.., TimeOfImpact(toi)): &Ray) -> Transform {
	let mut transform = Transform::from_translation((from + to) / 2.).looking_at(to, Vec3::Y);
	transform.scale.z = *toi;
	transform
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Beam, BeamConfig, LifeTime};
	use bevy::{
		app::{App, Update},
		color::{Color, LinearRgba},
		ecs::entity::Entity,
		hierarchy::BuildWorldChildren,
		math::{Ray3d, Vec3},
		prelude::default,
		render::view::{InheritedVisibility, ViewVisibility, Visibility},
		transform::{bundles::TransformBundle, components::Transform},
	};
	use bevy_rapier3d::pipeline::QueryFilter;
	use common::{
		components::GroundOffset,
		test_tools::utils::SingleThreadedApp,
		traits::cast_ray::TimeOfImpact,
	};
	use interactions::components::RayCaster;
	use std::time::Duration;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, execute_beam);
		app.add_event::<InteractionEvent<Ray>>();

		app
	}

	#[test]
	fn insert_ray_caster() {
		let mut app = setup();
		let source = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 0.))
			.id();
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 4.))
			.id();
		let beam = app
			.world_mut()
			.spawn((
				BeamConfig {
					range: 100.,
					..default()
				},
				BeamCommand { source, target },
			))
			.id();

		app.update();

		let ray_caster = app.world().entity(beam).get::<RayCaster>();

		assert_eq!(
			Some(&RayCaster {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Z,
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::from(QueryFilterFlags::EXCLUDE_SENSORS)
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
		let source = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 0.))
			.id();
		let target = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 4.),
				GroundOffset(Vec3::new(0., 1., 0.)),
			))
			.id();
		let beam = app
			.world_mut()
			.spawn((
				BeamConfig {
					range: 100.,
					..default()
				},
				BeamCommand { source, target },
			))
			.id();

		app.update();

		let ray_caster = app.world().entity(beam).get::<RayCaster>();

		assert_eq!(
			Some(&RayCaster {
				origin: Vec3::new(1., 0., 0.),
				direction: Vec3::new(0., 1., 4.).normalize().try_into().unwrap(),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::from(QueryFilterFlags::EXCLUDE_SENSORS)
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
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.),
				GroundOffset(Vec3::new(0., 1., 0.)),
			))
			.id();
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 4.))
			.id();
		let beam = app
			.world_mut()
			.spawn((
				BeamConfig {
					range: 100.,
					..default()
				},
				BeamCommand { source, target },
			))
			.id();

		app.update();

		let ray_caster = app.world().entity(beam).get::<RayCaster>();

		assert_eq!(
			Some(&RayCaster {
				origin: Vec3::new(1., 1., 0.),
				direction: Vec3::new(0., -1., 4.).normalize().try_into().unwrap(),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::from(QueryFilterFlags::EXCLUDE_SENSORS)
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			ray_caster
		);
	}

	#[test]
	fn spawn_beam_from_interaction() {
		let mut app = setup();
		let source = app.world_mut().spawn(GlobalTransform::default()).id();
		let beam = app
			.world_mut()
			.spawn((
				BeamConfig {
					color: Color::srgb(0.1, 0.2, 0.3),
					emissive: LinearRgba::new(1., 0.9, 0.8, 0.7),
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
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::Z,
				direction: Dir3::Y,
			},
			TimeOfImpact(10.),
		));

		app.update();

		let beam = app.world().entity(beam);

		assert_eq!(
			(
				Some(&Beam {
					from: Vec3::Z,
					to: Vec3::new(0., 10., 1.),
					damage: 42,
					color: Color::srgb(0.1, 0.2, 0.3),
					emissive: LinearRgba::new(1., 0.9, 0.8, 0.7)
				}),
				Some(&LifeTime(Duration::from_millis(100))),
			),
			(beam.get::<Beam>(), beam.get::<LifeTime>())
		);
	}

	#[test]
	fn set_spatial_bundle() {
		let mut app = setup();
		let source = app.world_mut().spawn(GlobalTransform::default()).id();
		let beam = app
			.world_mut()
			.spawn((
				BeamConfig::default(),
				BeamCommand {
					source,
					target: Entity::from_raw(default()),
				},
			))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(0., 1., 0.),
				direction: Dir3::X,
			},
			TimeOfImpact(10.),
		));

		app.update();

		let bundle = app
			.world()
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
		let source = app.world_mut().spawn(GlobalTransform::default()).id();
		let beam = app
			.world_mut()
			.spawn((
				BeamConfig::default(),
				BeamCommand {
					source,
					target: Entity::from_raw(default()),
				},
			))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(0., 1., 0.),
				direction: Dir3::X,
			},
			TimeOfImpact(10.),
		));

		app.update();

		app.update();

		let bundle = app
			.world()
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

	#[test]
	fn remove_beam_when_source_not_removed() {
		let mut app = setup();
		let source = app.world_mut().spawn(GlobalTransform::default()).id();
		let beam = app
			.world_mut()
			.spawn((
				BeamConfig::default(),
				BeamCommand {
					source,
					target: Entity::from_raw(default()),
				},
			))
			.id();
		let child = app.world_mut().spawn_empty().set_parent(beam).id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(0., 1., 0.),
				direction: Dir3::X,
			},
			TimeOfImpact(10.),
		));

		app.update();

		app.world_mut().entity_mut(source).despawn();

		app.update();

		let beam = app.world().get_entity(beam);
		let child = app.world().get_entity(child);

		assert_eq!((true, true), (beam.is_none(), child.is_none()));
	}
}
