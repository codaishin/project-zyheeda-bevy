use crate::{
	components::{RayCasterArgs, RayFilter, active_beam::ActiveBeam, blockable::Blockable},
	events::{InteractionEvent, Ray},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	tools::Units,
	traits::{
		accessors::get::{GetMut, TryApplyOn},
		cast_ray::TimeOfImpact,
		handles_interactions::InteractAble,
	},
	zyheeda_commands::ZyheedaCommands,
};

type BeamComponents<'a> = (
	Entity,
	&'a Blockable,
	&'a mut GlobalTransform,
	&'a mut Transform,
	Option<&'a ActiveBeam>,
);

impl ActiveBeam {
	pub(crate) fn execute(
		mut commands: ZyheedaCommands,
		mut ray_cast_events: EventReader<InteractionEvent<Ray>>,
		mut beams: Query<BeamComponents>,
	) {
		for (entity, Blockable(beam), global_transform, ..) in &beams {
			let InteractAble::Beam { emitter, .. } = beam else {
				continue;
			};

			match commands.get_mut(&emitter.mounted_on).map(|e| e.id()) {
				Some(mounted_on) => {
					update_ray_caster_args(
						&mut commands,
						entity,
						global_transform,
						mounted_on,
						emitter.range,
					);
				}
				None => despawn_beam(&mut commands, entity),
			}
		}

		for InteractionEvent(entity, ray) in ray_cast_events.read() {
			match beams.get_mut(*entity) {
				Err(_) => continue,
				Ok((entity, .., mut transform, None)) => {
					insert_active_beam(&mut commands, entity, transform.as_mut(), ray)
				}
				Ok((.., mut transform, Some(_beam))) => update_transform(transform.as_mut(), ray),
			}
		}
	}
}

fn update_ray_caster_args(
	commands: &mut ZyheedaCommands,
	entity: Entity,
	origin: &GlobalTransform,
	mounted_on: Entity,
	range: Units,
) {
	let Some(filter) = get_filter(mounted_on) else {
		return;
	};

	commands.try_apply_on(&entity, |mut e| {
		e.try_insert(RayCasterArgs {
			origin: origin.translation(),
			direction: origin.forward(),
			solid: true,
			filter,
			max_toi: TimeOfImpact(*range),
		});
	});
}

fn despawn_beam(commands: &mut ZyheedaCommands, entity: Entity) {
	commands.try_apply_on(&entity, |e| e.try_despawn());
}

fn get_filter(source: Entity) -> Option<RayFilter> {
	QueryFilter::default()
		.exclude_rigid_body(source)
		.try_into()
		.ok()
}

fn insert_active_beam(
	commands: &mut ZyheedaCommands,
	entity: Entity,
	transform: &mut Transform,
	ray: &Ray,
) {
	commands.try_apply_on(&entity, |mut e| {
		e.try_insert(ActiveBeam);
	});
	update_transform(transform, ray);
}

fn update_transform(transform: &mut Transform, ray: &Ray) {
	let TimeOfImpact(toi) = ray.1;
	transform.scale.z = toi;
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
	use testing::{SingleThreadedApp, assert_eq_approx};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_event::<InteractionEvent<Ray>>();
		app.add_systems(Update, ActiveBeam::execute);

		app
	}

	static MOUNTED_ON: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[test]
	fn insert_ray_caster_from_global_transform() {
		let mut app = setup();
		let mounted_on = app.world_mut().spawn(*MOUNTED_ON).id();
		let beam = app
			.world_mut()
			.spawn((
				GlobalTransform::from(Transform::from_xyz(2., 2., 2.).looking_to(Dir3::X, Vec3::Y)),
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y),
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

		assert_eq_approx!(
			Some(&RayCasterArgs {
				origin: Vec3::new(2., 2., 2.),
				direction: Dir3::X,
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(mounted_on)
					.try_into()
					.unwrap(),
			}),
			app.world().entity(beam).get::<RayCasterArgs>(),
			f32::EPSILON,
		);
	}

	#[test]
	fn insert_active_beam() {
		let mut app = setup();
		app.world_mut().spawn(*MOUNTED_ON);
		let beam = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y),
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
				direction: Dir3::Y,
			},
			TimeOfImpact(10.),
		));

		app.update();

		assert_eq!(
			(
				Some(&ActiveBeam),
				Some(
					&Transform::from_xyz(1., 0., 0.)
						.looking_to(Dir3::Z, Vec3::Y)
						.with_scale(Vec3::new(1., 1., 10.))
				)
			),
			(
				app.world().entity(beam).get::<ActiveBeam>(),
				app.world().entity(beam).get::<Transform>(),
			)
		);
	}

	#[test]
	fn update_active_beam() {
		let mut app = setup();
		app.world_mut().spawn(*MOUNTED_ON);
		let beam = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y),
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
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Y,
			},
			TimeOfImpact(5.),
		));

		app.update();

		assert_eq!(
			(
				Some(&ActiveBeam),
				Some(
					&Transform::from_xyz(1., 0., 0.)
						.looking_to(Dir3::Z, Vec3::Y)
						.with_scale(Vec3::new(1., 1., 5.))
				)
			),
			(
				app.world().entity(beam).get::<ActiveBeam>(),
				app.world().entity(beam).get::<Transform>(),
			)
		);
	}

	#[test]
	fn remove_beam_when_unmounted() {
		let mut app = setup();
		let mounted_on = app.world_mut().spawn(*MOUNTED_ON).id();
		let beam = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y),
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
