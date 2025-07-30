use crate::{
	components::{RayCasterArgs, RayFilter, active_beam::ActiveBeam, blockable::Blockable},
	events::{InteractionEvent, Ray},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	resources::persistent_entities::PersistentEntities,
	tools::Units,
	traits::{
		cast_ray::TimeOfImpact,
		handles_interactions::InteractAble,
		try_despawn::TryDespawn,
		try_insert_on::TryInsertOn,
	},
};

impl ActiveBeam {
	pub(crate) fn execute(
		mut commands: Commands,
		mut ray_cast_events: EventReader<InteractionEvent<Ray>>,
		mut persistent_entities: ResMut<PersistentEntities>,
		mut beams: Query<(
			Entity,
			&Blockable,
			&GlobalTransform,
			Option<&mut ActiveBeam>,
		)>,
	) {
		for (entity, Blockable(beam), origin, ..) in &beams {
			let InteractAble::Beam { emitter, .. } = beam else {
				continue;
			};

			match persistent_entities.get_entity(&emitter.mounted_on) {
				Some(mounted_on) => {
					update_ray_caster_args(
						&mut commands,
						entity,
						origin,
						mounted_on,
						emitter.range,
					);
				}
				None => despawn_beam(&mut commands, entity),
			}
		}

		for InteractionEvent(origin, ray) in ray_cast_events.read() {
			match beams.get_mut(*origin) {
				Err(_) => continue,
				Ok((entity, .., None)) => insert_active_beam(&mut commands, entity, ray),
				Ok((.., Some(beam))) => update_active_beam(beam, ray),
			}
		}
	}
}

fn update_ray_caster_args(
	commands: &mut Commands,
	entity: Entity,
	origin: &GlobalTransform,
	mounted_on: Entity,
	range: Units,
) {
	let Some(filter) = get_filter(mounted_on) else {
		return;
	};

	commands.try_insert_on(
		entity,
		RayCasterArgs {
			origin: origin.translation(),
			direction: origin.forward(),
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

fn insert_active_beam(commands: &mut Commands, entity: Entity, ray: &Ray) {
	let (source, target) = get_beam_range(&ray.0, ray.1);
	commands.try_insert_on(entity, ActiveBeam { source, target });
}

fn update_active_beam(mut beam: Mut<ActiveBeam>, ray: &Ray) {
	let (source, target) = get_beam_range(&ray.0, ray.1);
	beam.source = source;
	beam.target = target;
}

type SourceTranslation = Vec3;
type TargetTranslation = Vec3;

fn get_beam_range(
	ray: &Ray3d,
	TimeOfImpact(toi): TimeOfImpact,
) -> (SourceTranslation, TargetTranslation) {
	(ray.origin, ray.origin + *ray.direction * toi)
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
			handles_interactions::BeamEmitter,
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

	static MOUNTED_ON: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[test]
	fn insert_ray_caster() {
		let mut app = setup();
		let mounted_on = app.world_mut().spawn(*MOUNTED_ON).id();
		let beam = app
			.world_mut()
			.spawn((
				GlobalTransform::from(Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y)),
				Blockable(InteractAble::Beam {
					emitter: BeamEmitter {
						mounted_on: *MOUNTED_ON,
						range: Units::new(100.),
						insert_beam_model: |_| {},
					},
					blocked_by: default(),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&RayCasterArgs {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Z,
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(mounted_on)
					.try_into()
					.unwrap(),
			}),
			app.world().entity(beam).get::<RayCasterArgs>()
		);
	}

	#[test]
	fn insert_active_beam() {
		let mut app = setup();
		app.world_mut().spawn(*MOUNTED_ON);
		let beam = app
			.world_mut()
			.spawn((
				GlobalTransform::from(Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y)),
				Blockable(InteractAble::Beam {
					emitter: BeamEmitter {
						mounted_on: *MOUNTED_ON,
						range: Units::new(100.),
						insert_beam_model: |_| {},
					},
					blocked_by: default(),
				}),
			))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Z,
			},
			TimeOfImpact(10.),
		));

		app.update();

		assert_eq!(
			Some(&ActiveBeam {
				source: Vec3::new(1., 0., 0.),
				target: Vec3::new(1., 0., 10.)
			}),
			app.world().entity(beam).get::<ActiveBeam>(),
		);
	}

	#[test]
	fn update_active_beam() {
		let mut app = setup();
		app.world_mut().spawn(*MOUNTED_ON);
		let beam = app
			.world_mut()
			.spawn((
				GlobalTransform::from(Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y)),
				Blockable(InteractAble::Beam {
					emitter: BeamEmitter {
						mounted_on: *MOUNTED_ON,
						range: Units::new(100.),
						insert_beam_model: |_| {},
					},
					blocked_by: default(),
				}),
			))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Z,
			},
			TimeOfImpact(10.),
		));

		app.update();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(2., 0., 0.),
				direction: Dir3::Y,
			},
			TimeOfImpact(5.),
		));

		app.update();

		assert_eq!(
			Some(&ActiveBeam {
				source: Vec3::new(2., 0., 0.),
				target: Vec3::new(2., 5., 0.)
			}),
			app.world().entity(beam).get::<ActiveBeam>(),
		);
	}

	#[test]
	fn remove_beam_when_unmounted() {
		let mut app = setup();
		let mounted_on = app.world_mut().spawn(*MOUNTED_ON).id();
		let beam = app
			.world_mut()
			.spawn((
				GlobalTransform::from(Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y)),
				Blockable(InteractAble::Beam {
					emitter: BeamEmitter {
						mounted_on: *MOUNTED_ON,
						range: Units::new(100.),
						insert_beam_model: |_| {},
					},
					blocked_by: default(),
				}),
			))
			.id();
		let child = app.world_mut().spawn(ChildOf(beam)).id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Z,
			},
			TimeOfImpact(10.),
		));

		app.update();
		app.world_mut().entity_mut(mounted_on).despawn();
		app.update();

		let beam = app.world().get_entity(beam);
		let child = app.world().get_entity(child);

		assert_eq!((false, false), (beam.is_ok(), child.is_ok()));
	}
}
