use crate::{
	components::{RayCasterArgs, RayFilter, active_beam::ActiveBeam, blockable::Blockable},
	events::{InteractionEvent, Ray},
};
use bevy::prelude::*;
use common::{
	tools::Units,
	traits::{
		accessors::get::TryApplyOn,
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
			let InteractAble::Beam { range, .. } = beam else {
				continue;
			};

			update_ray_caster_args(&mut commands, entity, global_transform, *range);
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
	range: Units,
) {
	commands.try_apply_on(&entity, |mut e| {
		e.try_insert(RayCasterArgs {
			origin: origin.translation(),
			direction: origin.forward(),
			solid: true,
			filter: RayFilter::default(),
			max_toi: TimeOfImpact(*range),
		});
	});
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
	use common::traits::{
		cast_ray::TimeOfImpact,
		clamp_zero_positive::ClampZeroPositive,
		register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::{SingleThreadedApp, assert_eq_approx};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_event::<InteractionEvent<Ray>>();
		app.add_systems(Update, ActiveBeam::execute);

		app
	}

	#[test]
	fn insert_ray_caster_from_global_transform() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn((
				GlobalTransform::from(Transform::from_xyz(2., 2., 2.).looking_to(Dir3::X, Vec3::Y)),
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y),
				Blockable(InteractAble::Beam {
					range: Units::new(100.),
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
				filter: RayFilter::default(),
			}),
			app.world().entity(beam).get::<RayCasterArgs>(),
			f32::EPSILON,
		);
	}

	#[test]
	fn insert_active_beam() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y),
				Blockable(InteractAble::Beam {
					range: Units::new(100.),
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
		let beam = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y),
				Blockable(InteractAble::Beam {
					range: Units::new(100.),
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
}
