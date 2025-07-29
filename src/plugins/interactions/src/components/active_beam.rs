use super::{RayCasterArgs, RayFilter};
use crate::{
	components::blockable::Blockable,
	events::{InteractionEvent, Ray},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::ground_offset::GroundOffset,
	resources::persistent_entities::PersistentEntities,
	tools::Units,
	traits::{
		cast_ray::TimeOfImpact,
		handles_interactions::InteractAble,
		try_despawn::TryDespawn,
		try_insert_on::TryInsertOn,
	},
};

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
pub(crate) struct ActiveBeam {
	source: Vec3,
	target: Vec3,
}

impl ActiveBeam {
	pub(crate) fn execute(
		mut commands: Commands,
		mut ray_cast_events: EventReader<InteractionEvent<Ray>>,
		mut persistent_entities: ResMut<PersistentEntities>,
		beams: Query<(Entity, &Blockable, Option<&ActiveBeam>)>,
		transforms: Query<(&GlobalTransform, Option<&GroundOffset>)>,
	) {
		for (entity, Blockable(beam), ..) in &beams {
			let InteractAble::Beam { config: beam, .. } = beam else {
				continue;
			};

			match persistent_entities.get_entity(&beam.source) {
				Some(source) => {
					let Some(target) = persistent_entities.get_entity(&beam.target) else {
						continue;
					};
					insert_ray_caster_args(
						&mut commands,
						&transforms,
						entity,
						source,
						target,
						beam.range,
					);
				}
				None => despawn_beam(&mut commands, entity),
			}
		}

		for InteractionEvent(source, ray) in ray_cast_events.read() {
			match beams.get(*source) {
				Err(_) => continue,
				Ok((entity, .., None)) => spawn_beam(&mut commands, entity, ray),
				Ok((entity, .., Some(_beam))) => update_beam_transform(&mut commands, entity, ray),
			}
		}
	}
}

fn insert_ray_caster_args(
	commands: &mut Commands,
	transforms: &Query<(&GlobalTransform, Option<&GroundOffset>)>,
	entity: Entity,
	source: Entity,
	target: Entity,
	range: Units,
) {
	let Ok((source_transform, source_offset)) = transforms.get(source) else {
		return;
	};
	let Ok((target_transform, target_offset)) = transforms.get(target) else {
		return;
	};
	let Some(filter) = get_filter(source) else {
		return;
	};
	let origin = translation(source_transform, source_offset);
	let target = translation(target_transform, target_offset);
	let Ok(direction) = Dir3::new(target - origin) else {
		return;
	};

	commands.try_insert_on(
		entity,
		RayCasterArgs {
			origin,
			direction,
			solid: true,
			filter,
			max_toi: TimeOfImpact(*range),
		},
	);
}

fn despawn_beam(commands: &mut Commands, entity: Entity) {
	commands.try_despawn(entity)
}

fn get_filter(source: Entity) -> Option<RayFilter> {
	QueryFilter::default()
		.exclude_rigid_body(source)
		.try_into()
		.ok()
}

fn spawn_beam(commands: &mut Commands, entity: Entity, ray: &Ray) {
	let (source, target, transform) = unpack_beam_ray(ray);
	commands.try_insert_on(entity, (transform, ActiveBeam { source, target }));
}

fn update_beam_transform(commands: &mut Commands, entity: Entity, ray: &Ray) {
	let (.., transform) = unpack_beam_ray(ray);
	commands.try_insert_on(entity, transform);
}

fn translation(transform: &GlobalTransform, offset: Option<&GroundOffset>) -> Vec3 {
	transform.translation() + offset.map_or(Vec3::ZERO, |offset| offset.0)
}

type SourceTranslation = Vec3;
type TargetTranslation = Vec3;

fn get_beam_range(ray: &Ray3d, toi: f32) -> (SourceTranslation, TargetTranslation) {
	(ray.origin, ray.origin + *ray.direction * toi)
}

fn unpack_beam_ray(
	Ray(ray, TimeOfImpact(toi)): &Ray,
) -> (SourceTranslation, TargetTranslation, Transform) {
	let (source, target) = get_beam_range(ray, *toi);

	(
		source,
		target,
		Transform::from_translation((source + target) / 2.)
			.looking_at(target, Vec3::Y)
			.with_scale(Vec3::ONE.with_z(*toi)),
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::RayCasterArgs,
		events::{InteractionEvent, Ray},
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::{
			cast_ray::TimeOfImpact,
			clamp_zero_positive::ClampZeroPositive,
			handles_interactions::BeamConfig,
			register_persistent_entities::RegisterPersistentEntities,
		},
	};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_event::<InteractionEvent<Ray>>();
		app.add_systems(Update, ActiveBeam::execute);

		app
	}

	static SOURCE: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);
	static TARGET: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[test]
	fn insert_ray_caster() {
		let mut app = setup();
		let source = app
			.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), *SOURCE))
			.id();
		app.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 4.), *TARGET));
		let beam = app
			.world_mut()
			.spawn(Blockable(InteractAble::Beam {
				config: BeamConfig {
					source: *SOURCE,
					target: *TARGET,
					range: Units::new(100.),
				},
				blocked_by: default(),
			}))
			.id();

		app.update();

		assert_eq!(
			Some(&RayCasterArgs {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Z,
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			app.world().entity(beam).get::<RayCasterArgs>()
		);
	}

	#[test]
	fn insert_ray_caster_with_ground_offset_for_target() {
		let mut app = setup();
		let source = app
			.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), *SOURCE))
			.id();
		app.world_mut().spawn((
			GlobalTransform::from_xyz(1., 0., 4.),
			GroundOffset(Vec3::new(0., 1., 0.)),
			*TARGET,
		));
		let beam = app
			.world_mut()
			.spawn(Blockable(InteractAble::Beam {
				config: BeamConfig {
					source: *SOURCE,
					target: *TARGET,
					range: Units::new(100.),
				},
				blocked_by: default(),
			}))
			.id();

		app.update();

		assert_eq!(
			Some(&RayCasterArgs {
				origin: Vec3::new(1., 0., 0.),
				direction: Vec3::new(0., 1., 4.).normalize().try_into().unwrap(),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			app.world().entity(beam).get::<RayCasterArgs>()
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
				*SOURCE,
			))
			.id();
		app.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 4.), *TARGET));
		let beam = app
			.world_mut()
			.spawn(Blockable(InteractAble::Beam {
				config: BeamConfig {
					source: *SOURCE,
					target: *TARGET,
					range: Units::new(100.),
				},
				blocked_by: default(),
			}))
			.id();

		app.update();

		assert_eq!(
			Some(&RayCasterArgs {
				origin: Vec3::new(1., 1., 0.),
				direction: Vec3::new(0., -1., 4.).normalize().try_into().unwrap(),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			app.world().entity(beam).get::<RayCasterArgs>()
		);
	}

	#[test]
	fn spawn_beam_from_interaction() {
		let mut app = setup();
		app.world_mut().spawn((GlobalTransform::default(), *SOURCE));
		let beam = app
			.world_mut()
			.spawn(Blockable(InteractAble::Beam {
				config: BeamConfig {
					source: *SOURCE,
					target: *TARGET,
					range: default(),
				},
				blocked_by: default(),
			}))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::Z,
				direction: Dir3::Y,
			},
			TimeOfImpact(10.),
		));

		app.update();

		assert_eq!(
			Some(&ActiveBeam {
				source: Vec3::Z,
				target: Vec3::new(0., 10., 1.)
			}),
			app.world().entity(beam).get::<ActiveBeam>(),
		);
	}

	#[test]
	fn set_spatial_components() {
		let mut app = setup();
		app.world_mut().spawn((GlobalTransform::default(), *SOURCE));
		let beam = app
			.world_mut()
			.spawn(Blockable(InteractAble::Beam {
				config: BeamConfig {
					source: *SOURCE,
					target: *TARGET,
					range: default(),
				},
				blocked_by: default(),
			}))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(0., 1., 0.),
				direction: Dir3::X,
			},
			TimeOfImpact(10.),
		));

		app.update();

		assert_eq!(
			(
				Some(
					&Transform::from_xyz(5., 1., 0.)
						.looking_at(Vec3::new(10., 1., 0.), Vec3::Y)
						.with_scale(Vec3 {
							x: 1.,
							y: 1.,
							z: 10.
						})
				),
				Some(&Visibility::default()),
			),
			(
				app.world().entity(beam).get::<Transform>(),
				app.world().entity(beam).get::<Visibility>(),
			)
		);
	}

	#[test]
	fn update_transform_only_when_beam_component_already_present() {
		let mut app = setup();
		app.world_mut().spawn((GlobalTransform::default(), *SOURCE));
		let beam = app
			.world_mut()
			.spawn((
				Blockable(InteractAble::Beam {
					config: BeamConfig {
						source: *SOURCE,
						target: *TARGET,
						range: default(),
					},
					blocked_by: default(),
				}),
				ActiveBeam {
					source: default(),
					target: default(),
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

		assert_eq!(
			(
				Some(
					&Transform::from_xyz(5., 1., 0.)
						.looking_at(Vec3::new(10., 1., 0.), Vec3::Y)
						.with_scale(Vec3 {
							x: 1.,
							y: 1.,
							z: 10.
						})
				),
				Some(&ActiveBeam {
					source: default(),
					target: default()
				})
			),
			(
				app.world().entity(beam).get::<Transform>(),
				app.world().entity(beam).get::<ActiveBeam>(),
			),
		)
	}

	#[test]
	fn remove_beam_when_source_removed() {
		let mut app = setup();
		let source = app
			.world_mut()
			.spawn((GlobalTransform::default(), *SOURCE))
			.id();
		let beam = app
			.world_mut()
			.spawn(Blockable(InteractAble::Beam {
				config: BeamConfig {
					source: *SOURCE,
					target: *TARGET,
					range: default(),
				},
				blocked_by: default(),
			}))
			.id();
		let child = app.world_mut().spawn(ChildOf(beam)).id();
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

		assert_eq!((false, false), (beam.is_ok(), child.is_ok()));
	}
}
