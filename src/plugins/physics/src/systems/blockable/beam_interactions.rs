use crate::{
	components::{
		RayCasterArgs,
		RayFilter,
		blockable::Blockable,
		blocker_types::BlockerTypes,
		interaction_target::ColliderOfInteractionTarget,
		skill_transform::SkillTransforms,
	},
	events::BeamInteraction,
	traits::cast_ray::{
		CastRayContinuouslySorted,
		GetContinuousSortedRayCaster,
		InvalidIntersections,
		RayHit,
	},
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use bevy_rapier3d::plugin::ReadRapierContext;
use common::{
	errors::{ErrorData, Level},
	tools::Units,
	traits::handles_physics::{PhysicalObject, TimeOfImpact, physical_bodies::Blocker},
};
use std::{collections::HashSet, fmt::Debug};

impl Blockable {
	pub(crate) fn beam_interactions(
		beam_interactions: EventWriter<BeamInteraction>,
		cast_ray: StaticSystemParam<ReadRapierContext>,
		objects: Query<(Entity, &Self, &SkillTransforms, &GlobalTransform)>,
		transforms: Query<&mut Transform>,
		blockers: Query<&BlockerTypes>,
		interaction_colliders: Query<&ColliderOfInteractionTarget>,
	) -> Result<(), BeamError> {
		Self::beam_interactions_internal(
			beam_interactions,
			cast_ray,
			objects,
			transforms,
			blockers,
			interaction_colliders,
		)
	}

	fn beam_interactions_internal<TGetRayCaster, TCasterError>(
		mut beam_interactions: EventWriter<BeamInteraction>,
		cast_ray: StaticSystemParam<TGetRayCaster>,
		objects: Query<(Entity, &Self, &SkillTransforms, &GlobalTransform)>,
		mut transforms: Query<&mut Transform>,
		blockers: Query<&BlockerTypes>,
		interaction_colliders: Query<&ColliderOfInteractionTarget>,
	) -> Result<(), BeamError<TCasterError>>
	where
		TGetRayCaster: for<'w, 's> SystemParam<
			Item<'w, 's>: GetContinuousSortedRayCaster<RayCasterArgs, TError = TCasterError>,
		>,
	{
		let cast_ray = match cast_ray.get_continuous_sorted_ray_caster() {
			Ok(cast_ray) => cast_ray,
			Err(error) => return Err(BeamError::NoRayCaster(error)),
		};

		let mut invalid_beams = vec![];

		for (entity, Blockable(obj), skill_transforms, transform) in &objects {
			let PhysicalObject::Beam { range, blocked_by } = obj else {
				continue;
			};
			let ray = Self::ray(transform, *range);
			let hits = match cast_ray.cast_ray_continuously_sorted(&ray) {
				Ok(hits) => hits,
				Err(invalid_intersections) => {
					invalid_beams.push(InvalidBeam {
						entity,
						invalid_intersections,
					});
					continue;
				}
			};
			let mut toi = ray.max_toi;

			for hit in hits {
				beam_interactions.write(BeamInteraction {
					beam: entity,
					intersects: hit.entity,
				});

				if Self::blocked(&hit, blockers, interaction_colliders, blocked_by) {
					toi = hit.toi;
					break;
				}
			}

			for entity in skill_transforms.iter() {
				let Ok(mut transform) = transforms.get_mut(entity) else {
					continue;
				};

				// move beam center in the middle of both ends
				transform.translation.z = -*toi / 2.;

				// beams are y-aligned cylinders/capsules rotated forward, so we need to scale y direction
				transform.scale.y = *toi;
			}
		}

		if !invalid_beams.is_empty() {
			return Err(BeamError::InvalidBeams(invalid_beams));
		}

		Ok(())
	}

	fn ray(transform: &GlobalTransform, range: Units) -> RayCasterArgs {
		RayCasterArgs {
			origin: transform.translation(),
			direction: transform.forward(),
			max_toi: TimeOfImpact::from(range),
			solid: false,
			filter: RayFilter::default(),
		}
	}

	fn blocked(
		hit: &RayHit,
		blockers: Query<&BlockerTypes>,
		interaction_colliders: Query<&ColliderOfInteractionTarget>,
		blocked_by: &HashSet<Blocker>,
	) -> bool {
		let entity = match interaction_colliders.get(hit.entity) {
			Ok(ColliderOfInteractionTarget(entity)) => *entity,
			Err(_) => hit.entity,
		};

		let Ok(BlockerTypes(blockers)) = blockers.get(entity) else {
			return false;
		};

		!blockers.is_disjoint(blocked_by)
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum BeamError<TCasterError = BevyError> {
	NoRayCaster(TCasterError),
	InvalidBeams(Vec<InvalidBeam>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct InvalidBeam {
	entity: Entity,
	invalid_intersections: InvalidIntersections,
}

impl ErrorData for BeamError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Beam Error"
	}

	fn into_details(self) -> impl std::fmt::Display {
		match self {
			BeamError::NoRayCaster(error) => format!("No ray caster: {error:?}"),
			BeamError::InvalidBeams(beams) => format!("Invalid Beams: {beams:?}"),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::{
			blocker_types::BlockerTypes,
			interaction_target::ColliderOfInteractionTarget,
			skill_transform::SkillTransformOf,
		},
		events::BeamInteraction,
		traits::cast_ray::{CastRayContinuouslySorted, InvalidIntersections, RayHit},
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		errors::Unreachable,
		toi,
		tools::Units,
		traits::handles_physics::{PhysicalObject, physical_bodies::Blocker},
	};
	use macros::simple_mock;
	use mockall::predicate::eq;
	use std::collections::HashSet;
	use testing::{Mock, SingleThreadedApp, get_current_update_events};
	use zyheeda_core::prelude::Sorted;

	#[derive(Resource)]
	struct _GetRayCaster {
		mock: Mock_RayCaster,
	}

	impl GetContinuousSortedRayCaster<RayCasterArgs> for Res<'_, _GetRayCaster> {
		type TError = Unreachable;

		type TRayCaster<'a>
			= &'a Mock_RayCaster
		where
			Self: 'a;

		fn get_continuous_sorted_ray_caster(
			&self,
		) -> std::result::Result<Self::TRayCaster<'_>, Self::TError> {
			Ok(&self.mock)
		}
	}

	simple_mock! {
		_RayCaster {}
		impl CastRayContinuouslySorted<RayCasterArgs> for _RayCaster {
			fn cast_ray_continuously_sorted(
				&self,
				ray: &RayCasterArgs,
			) -> Result<Sorted<RayHit>, InvalidIntersections>;
		}
	}

	impl CastRayContinuouslySorted<RayCasterArgs> for &'_ Mock_RayCaster {
		fn cast_ray_continuously_sorted(
			&self,
			ray: &RayCasterArgs,
		) -> Result<Sorted<RayHit>, InvalidIntersections> {
			(*self).cast_ray_continuously_sorted(ray)
		}
	}

	fn setup(new_mock: fn(&mut World) -> Mock_RayCaster) -> App {
		let mut app = App::new().single_threaded(Update);
		let mock = new_mock(app.world_mut());

		app.add_event::<BeamInteraction>();
		app.insert_resource(_GetRayCaster { mock });

		app
	}

	mod cast_ray_usage {
		use super::*;

		#[test]
		fn call_with_proper_args() -> Result<(), RunSystemError> {
			let mut app = setup(|_| Mock_RayCaster::new_mock(assert_call_args));
			app.world_mut().spawn((
				GlobalTransform::from(
					Transform::from_xyz(1., 2., 3.).looking_to(Dir3::NEG_Y, Vec3::Y),
				),
				Blockable(PhysicalObject::Beam {
					range: Units::from(11000.),
					blocked_by: HashSet::from([]),
				}),
			));

			_ = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<Res<_GetRayCaster>, Unreachable>,
			)?;

			fn assert_call_args(mock: &mut Mock_RayCaster) {
				mock.expect_cast_ray_continuously_sorted()
					.once()
					.with(eq(RayCasterArgs {
						origin: Vec3::new(1., 2., 3.),
						direction: Dir3::NEG_Y,
						max_toi: toi!(11000.),
						solid: false,
						filter: RayFilter::default(),
					}))
					.return_const(Ok(Sorted::from([])));
			}
			Ok(())
		}
	}

	mod skill_transforms {
		use super::*;

		#[test]
		fn update_to_reach_max_length() -> Result<(), RunSystemError> {
			let mut app = setup(|_| {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: Entity::from_raw(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: Entity::from_raw(41),
								toi: toi!(110.),
							},
							RayHit {
								entity: Entity::from_raw(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn(Blockable(PhysicalObject::Beam {
					range: Units::from(11000.),
					blocked_by: HashSet::from([]),
				}))
				.id();
			let skill_transform = app.world_mut().spawn(SkillTransformOf(entity)).id();

			_ = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<Res<_GetRayCaster>, Unreachable>,
			)?;

			assert_eq!(
				Some(&Transform {
					translation: Vec3::ZERO.with_z(-5500.),
					scale: Vec3::ONE.with_y(11000.),
					..default()
				}),
				app.world().entity(skill_transform).get::<Transform>(),
			);
			Ok(())
		}

		#[test]
		fn update_to_reach_first_block() -> Result<(), RunSystemError> {
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn(BlockerTypes(HashSet::from([
							Blocker::Force,
							Blocker::Physical,
						])))
						.id();
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: Entity::from_raw(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: blocker,
								toi: toi!(110.),
							},
							RayHit {
								entity: Entity::from_raw(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn(Blockable(PhysicalObject::Beam {
					range: Units::from(11000.),
					blocked_by: HashSet::from([Blocker::Force, Blocker::Character]),
				}))
				.id();
			let skill_transform = app.world_mut().spawn(SkillTransformOf(entity)).id();

			_ = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<Res<_GetRayCaster>, Unreachable>,
			)?;

			assert_eq!(
				Some(&Transform {
					translation: Vec3::ZERO.with_z(-55.),
					scale: Vec3::ONE.with_y(110.),
					..default()
				}),
				app.world().entity(skill_transform).get::<Transform>(),
			);
			Ok(())
		}

		#[test]
		fn update_to_reach_first_block_via_interaction_collider() -> Result<(), RunSystemError> {
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn(BlockerTypes(HashSet::from([
							Blocker::Force,
							Blocker::Physical,
						])))
						.id();
					let collider = world.spawn(ColliderOfInteractionTarget(blocker)).id();
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: Entity::from_raw(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: collider,
								toi: toi!(110.),
							},
							RayHit {
								entity: Entity::from_raw(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn(Blockable(PhysicalObject::Beam {
					range: Units::from(11000.),
					blocked_by: HashSet::from([Blocker::Force, Blocker::Character]),
				}))
				.id();
			let skill_transform = app.world_mut().spawn(SkillTransformOf(entity)).id();

			_ = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<Res<_GetRayCaster>, Unreachable>,
			)?;

			assert_eq!(
				Some(&Transform {
					translation: Vec3::ZERO.with_z(-55.),
					scale: Vec3::ONE.with_y(110.),
					..default()
				}),
				app.world().entity(skill_transform).get::<Transform>(),
			);
			Ok(())
		}

		#[test]
		fn update_to_reach_max_length_if_blockers_do_not_match() -> Result<(), RunSystemError> {
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn(BlockerTypes(HashSet::from([Blocker::Physical])))
						.id();
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: Entity::from_raw(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: blocker,
								toi: toi!(110.),
							},
							RayHit {
								entity: Entity::from_raw(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn(Blockable(PhysicalObject::Beam {
					range: Units::from(11000.),
					blocked_by: HashSet::from([Blocker::Force, Blocker::Character]),
				}))
				.id();
			let skill_transform = app.world_mut().spawn(SkillTransformOf(entity)).id();

			_ = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<Res<_GetRayCaster>, Unreachable>,
			)?;

			assert_eq!(
				Some(&Transform {
					translation: Vec3::ZERO.with_z(-5500.),
					scale: Vec3::ONE.with_y(11000.),
					..default()
				}),
				app.world().entity(skill_transform).get::<Transform>(),
			);
			Ok(())
		}

		#[test]
		fn update_to_max_range_if_not_blocked() -> Result<(), RunSystemError> {
			let mut app = setup(|_| {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([])));
				})
			});
			let entity = app
				.world_mut()
				.spawn(Blockable(PhysicalObject::Beam {
					range: Units::from(11000.),
					blocked_by: HashSet::from([Blocker::Force, Blocker::Character]),
				}))
				.id();
			let skill_transform = app.world_mut().spawn(SkillTransformOf(entity)).id();

			_ = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<Res<_GetRayCaster>, Unreachable>,
			)?;

			assert_eq!(
				Some(&Transform {
					translation: Vec3::ZERO.with_z(-5500.),
					scale: Vec3::ONE.with_y(11000.),
					..default()
				}),
				app.world().entity(skill_transform).get::<Transform>(),
			);
			Ok(())
		}
	}

	mod beam_interaction_events {
		use super::*;

		#[test]
		fn send_event_for_each_hit() -> Result<(), RunSystemError> {
			let mut app = setup(|_| {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: Entity::from_raw(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: Entity::from_raw(41),
								toi: toi!(110.),
							},
							RayHit {
								entity: Entity::from_raw(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn(Blockable(PhysicalObject::Beam {
					range: Units::from(11000.),
					blocked_by: HashSet::from([]),
				}))
				.id();

			_ = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<Res<_GetRayCaster>, Unreachable>,
			)?;

			assert_eq!(
				vec![
					&BeamInteraction {
						beam: entity,
						intersects: Entity::from_raw(42)
					},
					&BeamInteraction {
						beam: entity,
						intersects: Entity::from_raw(41)
					},
					&BeamInteraction {
						beam: entity,
						intersects: Entity::from_raw(40)
					},
				],
				get_current_update_events!(app, BeamInteraction).collect::<Vec<_>>(),
			);
			Ok(())
		}

		#[test]
		fn send_event_for_each_hit_until_blocked() -> Result<(), RunSystemError> {
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn(BlockerTypes(HashSet::from([
							Blocker::Physical,
							Blocker::Force,
						])))
						.id();

					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: Entity::from_raw(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: blocker,
								toi: toi!(110.),
							},
							RayHit {
								entity: Entity::from_raw(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn(Blockable(PhysicalObject::Beam {
					range: Units::from(11000.),
					blocked_by: HashSet::from([Blocker::Physical, Blocker::Character]),
				}))
				.id();

			_ = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<Res<_GetRayCaster>, Unreachable>,
			)?;

			let blocker = app
				.world()
				.iter_entities()
				.find(|e| e.contains::<BlockerTypes>())
				.unwrap();
			assert_eq!(
				vec![
					&BeamInteraction {
						beam: entity,
						intersects: Entity::from_raw(42)
					},
					&BeamInteraction {
						beam: entity,
						intersects: blocker.id()
					},
				],
				get_current_update_events!(app, BeamInteraction).collect::<Vec<_>>(),
			);
			Ok(())
		}
	}

	mod result {
		use super::*;

		#[test]
		fn return_invalid_intersections_error() -> Result<(), RunSystemError> {
			let mut app = setup(|_| {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Err(InvalidIntersections(vec![Vec3::new(1., 2., 3.)])));
				})
			});
			let entity = app
				.world_mut()
				.spawn(Blockable(PhysicalObject::Beam {
					range: Units::from(11000.),
					blocked_by: HashSet::from([]),
				}))
				.id();

			let result = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<Res<_GetRayCaster>, Unreachable>,
			)?;

			assert_eq!(
				Err(BeamError::InvalidBeams(vec![InvalidBeam {
					entity,
					invalid_intersections: InvalidIntersections(vec![Vec3::new(1., 2., 3.)])
				}])),
				result,
			);
			Ok(())
		}

		#[test]
		fn return_ok() -> Result<(), RunSystemError> {
			let mut app = setup(|_| {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([])));
				})
			});
			app.world_mut().spawn(Blockable(PhysicalObject::Beam {
				range: Units::from(11000.),
				blocked_by: HashSet::from([]),
			}));

			let result = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<Res<_GetRayCaster>, Unreachable>,
			)?;

			assert!(result.is_ok());
			Ok(())
		}

		#[derive(SystemParam)]
		struct _FaultyRayCaster;

		impl GetContinuousSortedRayCaster<RayCasterArgs> for _FaultyRayCaster {
			type TError = _CasterError;
			type TRayCaster<'a>
				= _NoCaster
			where
				Self: 'a;

			fn get_continuous_sorted_ray_caster(
				&self,
			) -> Result<Self::TRayCaster<'_>, Self::TError> {
				Err(_CasterError)
			}
		}

		#[derive(Debug, PartialEq)]
		struct _CasterError;

		struct _NoCaster;

		impl CastRayContinuouslySorted<RayCasterArgs> for _NoCaster {
			fn cast_ray_continuously_sorted(
				&self,
				_: &RayCasterArgs,
			) -> Result<Sorted<RayHit>, InvalidIntersections> {
				panic!("DO NOT USE")
			}
		}

		#[test]
		fn return_no_ray_caster() -> Result<(), RunSystemError> {
			let mut app = setup(|_| {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([])));
				})
			});
			app.world_mut().spawn(Blockable(PhysicalObject::Beam {
				range: Units::from(11000.),
				blocked_by: HashSet::from([]),
			}));

			let result = app.world_mut().run_system_once(
				Blockable::beam_interactions_internal::<_FaultyRayCaster, _CasterError>,
			)?;

			assert_eq!(Err(BeamError::NoRayCaster(_CasterError)), result);
			Ok(())
		}
	}
}
