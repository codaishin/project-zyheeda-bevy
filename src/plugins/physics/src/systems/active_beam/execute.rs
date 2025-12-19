use crate::{
	components::{RayCasterArgs, RayFilter, active_beam::ActiveBeam, blockable::Blockable},
	events::{InteractionEvent, Ray},
};
use bevy::prelude::*;
use common::{
	tools::Units,
	traits::{
		accessors::get::TryApplyOn,
		handles_physics::{PhysicalObject, TimeOfImpact},
	},
	zyheeda_commands::ZyheedaCommands,
};
use core::f32;

impl ActiveBeam {
	pub(crate) fn execute(
		mut commands: ZyheedaCommands,
		mut ray_cast_events: EventReader<InteractionEvent<Ray>>,
		mut beams: Query<(
			Entity,
			&Blockable,
			&GlobalTransform,
			Option<&mut ActiveBeam>,
		)>,
	) {
		for (entity, Blockable(beam), global_transform, ..) in &beams {
			let PhysicalObject::Beam { range, .. } = beam else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(RayCasterArgs {
					origin: global_transform.translation(),
					direction: global_transform.forward(),
					solid: false,
					filter: RayFilter::default(),
					max_toi: TimeOfImpact::from(*range),
				});
			});
		}

		for InteractionEvent(entity, Ray(.., toi)) in ray_cast_events.read() {
			let length = Units::from(f32::max(**toi, f32::EPSILON));

			match beams.get_mut(*entity) {
				Err(_) => continue,
				Ok((entity, .., None)) => {
					commands.try_apply_on(&entity, |mut e| {
						e.try_insert(ActiveBeam { length });
					});
				}
				Ok((.., Some(mut beam))) => {
					beam.length = length;
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::RayCasterArgs,
		events::{InteractionEvent, Ray},
	};
	use common::{toi, traits::register_persistent_entities::RegisterPersistentEntities};
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
				Blockable(PhysicalObject::Beam {
					range: Units::from(100.),
					blocked_by: default(),
				}),
			))
			.id();

		app.update();

		assert_eq_approx!(
			Some(&RayCasterArgs {
				origin: Vec3::new(2., 2., 2.),
				direction: Dir3::X,
				max_toi: toi!(100.),
				solid: false,
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
				Blockable(PhysicalObject::Beam {
					range: Units::from(100.),
					blocked_by: default(),
				}),
			))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Y,
			},
			toi!(10.),
		));

		app.update();

		assert_eq!(
			Some(&ActiveBeam {
				length: Units::from(10.)
			}),
			app.world().entity(beam).get::<ActiveBeam>(),
		);
	}

	#[test]
	fn insert_active_beam_min_length_epsilon_when_toi_zero() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y),
				Blockable(PhysicalObject::Beam {
					range: Units::from(100.),
					blocked_by: default(),
				}),
			))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Y,
			},
			toi!(0.),
		));

		app.update();

		assert_eq!(
			Some(&ActiveBeam {
				length: Units::EPSILON
			}),
			app.world().entity(beam).get::<ActiveBeam>(),
		);
	}

	#[test]
	fn update_active_beam() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y),
				Blockable(PhysicalObject::Beam {
					range: Units::from(100.),
					blocked_by: default(),
				}),
			))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Z,
			},
			toi!(10.),
		));

		app.update();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Y,
			},
			toi!(5.),
		));

		app.update();

		assert_eq!(
			Some(&ActiveBeam {
				length: Units::from(5.)
			}),
			app.world().entity(beam).get::<ActiveBeam>(),
		);
	}

	#[test]
	fn update_active_beam_min_length_epsilon_when_toi_zero() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Vec3::Y),
				Blockable(PhysicalObject::Beam {
					range: Units::from(100.),
					blocked_by: default(),
				}),
			))
			.id();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Z,
			},
			toi!(5.),
		));

		app.update();
		app.world_mut().send_event(InteractionEvent::of(beam).ray(
			Ray3d {
				origin: Vec3::new(1., 0., 0.),
				direction: Dir3::Y,
			},
			toi!(0.),
		));

		app.update();

		assert_eq!(
			Some(&ActiveBeam {
				length: Units::EPSILON
			}),
			app.world().entity(beam).get::<ActiveBeam>(),
		);
	}
}
