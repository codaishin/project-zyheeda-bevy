use crate::{
	components::{
		blockable::Blockable,
		blocker_types::BlockerTypes,
		cast_rays::{CastRayFor, CastRays, RayCastResult, RayCasterArgs, RayFilter},
		collider::{AGENTS_GROUP, ColliderOf, RAY_GROUP, SKILLS_GROUP, TERRAIN_GROUP},
		collision_domains::Physical,
	},
	traits::ray_cast::{
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
use bevy_rapier3d::prelude::*;
use common::prelude::*;
use std::collections::HashSet;

const BIAS: f32 = 0.01;

impl CastRays {
	pub(crate) fn for_beams(
		objects: Query<(Entity, &Blockable, &mut Self, &GlobalTransform)>,
		blockers: Query<&BlockerTypes>,
		contacts: Query<(Option<&ColliderOf>, &Physical)>,
		cast_ray: StaticSystemParam<ReadRapierContext>,
	) -> Result<(), BeamError> {
		Self::for_beams_internal(objects, blockers, contacts, cast_ray)
	}

	fn for_beams_internal<TGetRayCaster, TCasterError>(
		objects: Query<(Entity, &Blockable, &mut Self, &GlobalTransform)>,
		blockers: Query<&BlockerTypes>,
		contacts: Query<(Option<&ColliderOf>, &Physical)>,
		cast_ray: StaticSystemParam<TGetRayCaster>,
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

		for (entity, Blockable(obj), mut cast_rays, transform) in objects {
			let PhysicalObject::Beam { range, blocked_by } = obj else {
				continue;
			};
			let args = Self::beam_ray_args(transform, *range);
			let hits = match cast_ray.cast_ray_continuously_sorted(&args) {
				Ok(hits) => hits,
				Err(invalid_intersections) => {
					invalid_beams.push(InvalidBeam {
						entity,
						invalid_intersections,
					});
					continue;
				}
			};
			let mut toi = args.max_toi;
			let is_blocked = |hit: &RayHit| Self::beam_blocked(hit, blockers, blocked_by, contacts);

			if let Some(blocked) = hits.into_iter().find(is_blocked) {
				let new_toi = (*blocked.toi + BIAS).clamp(0., *args.max_toi);
				toi = TimeOfImpact::from(Units::from(new_toi));
			}

			cast_rays
				.results
				.insert(CastRayFor::Beam, RayCastResult { args, toi });
		}

		if !invalid_beams.is_empty() {
			return Err(BeamError::InvalidBeams(invalid_beams));
		}

		Ok(())
	}

	fn beam_ray_args(transform: &GlobalTransform, range: Units) -> RayCasterArgs {
		RayCasterArgs {
			origin: transform.translation(),
			direction: transform.forward(),
			max_toi: TimeOfImpact::from(range),
			solid: true,
			filter: RayFilter {
				groups: Some(CollisionGroups {
					memberships: RAY_GROUP,
					filters: TERRAIN_GROUP | AGENTS_GROUP | SKILLS_GROUP,
				}),
				..default()
			},
		}
	}

	fn beam_blocked(
		hit: &RayHit,
		blockers: Query<&BlockerTypes>,
		blocked_by: &HashSet<Blocker>,
		contacts: Query<(Option<&ColliderOf>, &Physical)>,
	) -> bool {
		let entity = match contacts.get(hit.entity) {
			Ok((Some(ColliderOf(root)), Physical::Contact)) => *root,
			Ok((None, Physical::Contact)) => hit.entity,
			_ => return false,
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
	use super::*;
	use crate::{
		components::{
			blockable::Blockable,
			cast_rays::{CastRayFor, RayCastResult},
			collision_domains::Physical,
		},
		traits::ray_cast::{CastRayContinuouslySorted, InvalidIntersections, RayHit},
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{errors::Unreachable, tools::Units, traits::handles_physics::PhysicalObject};
	use macros::simple_mock;
	use mockall::predicate::eq;
	use std::collections::{HashMap, HashSet};
	use testing::{Mock, SingleThreadedApp, assert_eq_approx, fake_entity};
	use zyheeda_core::collections::sorted::Sorted;

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

		app.insert_resource(_GetRayCaster { mock });

		app
	}

	fn ray_args(origin: Vec3, direction: Dir3) -> RayCasterArgs {
		RayCasterArgs {
			origin,
			direction,
			max_toi: toi!(11000.),
			solid: true,
			filter: RayFilter {
				groups: Some(CollisionGroups {
					memberships: RAY_GROUP,
					filters: TERRAIN_GROUP | AGENTS_GROUP | SKILLS_GROUP,
				}),
				..default()
			},
		}
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
				Physical::Contact,
				Blockable(PhysicalObject::Beam {
					range: Units::from(11000.),
					blocked_by: HashSet::from([]),
				}),
			));

			_ = app
				.world_mut()
				.run_system_once(CastRays::for_beams_internal::<Res<_GetRayCaster>, Unreachable>)?;

			fn assert_call_args(mock: &mut Mock_RayCaster) {
				mock.expect_cast_ray_continuously_sorted()
					.once()
					.with(eq(ray_args(Vec3::new(1., 2., 3.), Dir3::NEG_Y)))
					.return_const(Ok(Sorted::from([])));
			}
			Ok(())
		}
	}

	mod result {
		use super::*;

		#[test]
		fn no_block() -> Result<(), RunSystemError> {
			let mut app = setup(|_| {
				Mock_RayCaster::new_mock(|mock| {
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: fake_entity!(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: fake_entity!(41),
								toi: toi!(110.),
							},
							RayHit {
								entity: fake_entity!(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn((
					Blockable(PhysicalObject::Beam {
						range: Units::from(11000.),
						blocked_by: HashSet::from([]),
					}),
					Physical::Contact,
				))
				.id();

			_ = app
				.world_mut()
				.run_system_once(CastRays::for_beams_internal::<Res<_GetRayCaster>, Unreachable>)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(11000.),
						}
					)])
				}),
				app.world().entity(entity).get::<CastRays>(),
				0.05,
			);
			Ok(())
		}

		#[test]
		fn reach_first_block() -> Result<(), RunSystemError> {
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn((
							BlockerTypes(HashSet::from([Blocker::Force, Blocker::Physical])),
							Physical::Contact,
						))
						.id();
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: fake_entity!(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: blocker,
								toi: toi!(110.),
							},
							RayHit {
								entity: fake_entity!(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn((
					Blockable(PhysicalObject::Beam {
						range: Units::from(11000.),
						blocked_by: HashSet::from([Blocker::Force, Blocker::Character]),
					}),
					Physical::Contact,
				))
				.id();

			_ = app
				.world_mut()
				.run_system_once(CastRays::for_beams_internal::<Res<_GetRayCaster>, Unreachable>)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(110.),
						}
					)])
				}),
				app.world().entity(entity).get::<CastRays>(),
				0.05,
			);
			Ok(())
		}

		#[test]
		fn reach_first_block_via_child_collider() -> Result<(), RunSystemError> {
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn((BlockerTypes(HashSet::from([
							Blocker::Force,
							Blocker::Physical,
						])),))
						.id();
					let collider = world.spawn((ColliderOf(blocker), Physical::Contact)).id();
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: fake_entity!(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: collider,
								toi: toi!(110.),
							},
							RayHit {
								entity: fake_entity!(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn((
					Blockable(PhysicalObject::Beam {
						range: Units::from(11000.),
						blocked_by: HashSet::from([Blocker::Force, Blocker::Character]),
					}),
					Physical::Contact,
				))
				.id();

			_ = app
				.world_mut()
				.run_system_once(CastRays::for_beams_internal::<Res<_GetRayCaster>, Unreachable>)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(110.),
						}
					)])
				}),
				app.world().entity(entity).get::<CastRays>(),
				0.05,
			);
			Ok(())
		}

		#[test]
		fn reach_max_toi_on_blocker_mismatch() -> Result<(), RunSystemError> {
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn((
							BlockerTypes(HashSet::from([Blocker::Physical])),
							Physical::Contact,
						))
						.id();
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: fake_entity!(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: blocker,
								toi: toi!(110.),
							},
							RayHit {
								entity: fake_entity!(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn((
					Blockable(PhysicalObject::Beam {
						range: Units::from(11000.),
						blocked_by: HashSet::from([Blocker::Force, Blocker::Character]),
					}),
					Physical::Contact,
				))
				.id();

			_ = app
				.world_mut()
				.run_system_once(CastRays::for_beams_internal::<Res<_GetRayCaster>, Unreachable>)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(11000.),
						}
					)])
				}),
				app.world().entity(entity).get::<CastRays>(),
				0.05,
			);
			Ok(())
		}

		#[test]
		fn reach_max_toi_when_blocker_not_physical_contact() -> Result<(), RunSystemError> {
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn((
							BlockerTypes(HashSet::from([Blocker::Physical])),
							Physical::Projection,
						))
						.id();
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([
							RayHit {
								entity: fake_entity!(42),
								toi: toi!(11.),
							},
							RayHit {
								entity: blocker,
								toi: toi!(110.),
							},
							RayHit {
								entity: fake_entity!(40),
								toi: toi!(1100.),
							},
						])));
				})
			});
			let entity = app
				.world_mut()
				.spawn((
					Blockable(PhysicalObject::Beam {
						range: Units::from(11000.),
						blocked_by: HashSet::from([Blocker::Force, Blocker::Character]),
					}),
					Physical::Contact,
				))
				.id();

			_ = app
				.world_mut()
				.run_system_once(CastRays::for_beams_internal::<Res<_GetRayCaster>, Unreachable>)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(11000.),
						}
					)])
				}),
				app.world().entity(entity).get::<CastRays>(),
				0.05,
			);
			Ok(())
		}

		#[test]
		fn clamp_toi_to_max_toi() -> Result<(), RunSystemError> {
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn((
							BlockerTypes(HashSet::from([Blocker::Force])),
							Physical::Projection,
						))
						.id();
					mock.expect_cast_ray_continuously_sorted()
						.return_const(Ok(Sorted::from([RayHit {
							entity: blocker,
							toi: toi!(110000.),
						}])));
				})
			});
			let entity = app
				.world_mut()
				.spawn((
					Blockable(PhysicalObject::Beam {
						range: Units::from(11000.),
						blocked_by: HashSet::from([Blocker::Force, Blocker::Character]),
					}),
					Physical::Contact,
				))
				.id();

			_ = app
				.world_mut()
				.run_system_once(CastRays::for_beams_internal::<Res<_GetRayCaster>, Unreachable>)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(11000.),
						}
					)])
				}),
				app.world().entity(entity).get::<CastRays>(),
				0.05,
			);
			Ok(())
		}
	}

	mod error {
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

			let result = app
				.world_mut()
				.run_system_once(CastRays::for_beams_internal::<Res<_GetRayCaster>, Unreachable>)?;

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

			let result = app
				.world_mut()
				.run_system_once(CastRays::for_beams_internal::<Res<_GetRayCaster>, Unreachable>)?;

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

			let result = app
				.world_mut()
				.run_system_once(CastRays::for_beams_internal::<_FaultyRayCaster, _CasterError>)?;

			assert_eq!(Err(BeamError::NoRayCaster(_CasterError)), result);
			Ok(())
		}
	}
}
