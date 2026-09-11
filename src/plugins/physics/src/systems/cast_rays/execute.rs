use crate::{
	components::{
		blockable::Blockable,
		blocker_types::BlockerTypes,
		cast_rays::{CastRayFor, CastRays, RayCastResult, RayCasterArgs, RayFilter},
		collider::{AGENTS_GROUP, ColliderOf, RAY_GROUP, SKILLS_GROUP, TERRAIN_GROUP},
		collision_domains::Physical,
		projectile::Projectile,
	},
	traits::ray_cast::{
		CastRayContinuouslySorted,
		GetContinuousSortedRayCaster,
		InvalidIntersections,
		RayHit,
	},
};
use bevy::{
	ecs::{
		query::{QueryData, ROQueryItem},
		system::{StaticSystemParam, SystemParam},
	},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::prelude::*;
use std::{collections::HashSet, time::Duration};

const BIAS: f32 = 0.01;

impl CastRays {
	pub(crate) fn for_beams() -> BeamStrategy {
		const { BeamStrategy }
	}

	pub(crate) fn to_prevent_tunneling(In(delta): In<Duration>) -> PreventTunnelingStrategy {
		PreventTunnelingStrategy {
			delta_secs: delta.as_secs_f32(),
		}
	}

	pub(crate) fn execute<TStrategy>(
		strategy: In<TStrategy>,
		strategy_args: Query<TStrategy::TQuery>,
		objects: Query<(Entity, &mut Self, &GlobalTransform)>,
		blockers: Query<&BlockerTypes>,
		contacts: Query<(Option<&ColliderOf>, &Physical)>,
		cast_ray: StaticSystemParam<ReadRapierContext>,
	) -> Result<(), RayError>
	where
		TStrategy: CastRayStrategy,
	{
		Self::execute_internal(
			strategy,
			strategy_args,
			objects,
			blockers,
			contacts,
			cast_ray,
		)
	}

	fn execute_internal<TStrategy, TGetRayCaster, TCasterError>(
		In(strategy): In<TStrategy>,
		strategy_args: Query<TStrategy::TQuery>,
		objects: Query<(Entity, &mut Self, &GlobalTransform)>,
		blockers: Query<&BlockerTypes>,
		contacts: Query<(Option<&ColliderOf>, &Physical)>,
		cast_ray: StaticSystemParam<TGetRayCaster>,
	) -> Result<(), RayError<TCasterError>>
	where
		TStrategy: CastRayStrategy,
		TGetRayCaster: for<'w, 's> SystemParam<
			Item<'w, 's>: GetContinuousSortedRayCaster<RayCasterArgs, TError = TCasterError>,
		>,
	{
		let cast_ray = match cast_ray.get_continuous_sorted_ray_caster() {
			Ok(cast_ray) => cast_ray,
			Err(error) => return Err(RayError::NoRayCaster(error)),
		};

		let mut invalid_rays = vec![];

		for (entity, mut cast_rays, transform) in objects {
			let Ok(strategy_args) = strategy_args.get(entity) else {
				continue;
			};
			let Some(strategy) = strategy.instance(strategy_args) else {
				continue;
			};
			let args = Self::ray_caster_args(transform, strategy.toi, strategy.origin_offset);
			let hits = match cast_ray.cast_ray_continuously_sorted(&args) {
				Ok(hits) => hits,
				Err(invalid_intersections) => {
					invalid_rays.push(InvalidRay {
						entity,
						invalid_intersections,
					});
					continue;
				}
			};
			let mut toi = args.max_toi;
			let mut hit = None;
			let is_blocked =
				|hit: &RayHit| Self::ray_blocked(hit, blockers, strategy.blocked_by, contacts);

			if let Some(blocked) = hits.into_iter().find(is_blocked) {
				let new_toi = (*blocked.toi + BIAS).clamp(0., *args.max_toi);
				toi = TimeOfImpact::from(Units::from(new_toi));
				hit = Some(blocked.entity);
			}

			cast_rays
				.results
				.insert(strategy.cast_for, RayCastResult { args, toi, hit });
		}

		if !invalid_rays.is_empty() {
			return Err(RayError::InvalidRay(invalid_rays));
		}

		Ok(())
	}

	fn ray_caster_args(
		transform: &GlobalTransform,
		max_toi: TimeOfImpact,
		offset: Units,
	) -> RayCasterArgs {
		RayCasterArgs {
			origin: transform.translation() + transform.forward() * *offset,
			direction: transform.forward(),
			max_toi,
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

	fn ray_blocked(
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
pub(crate) enum RayError<TCasterError = BevyError> {
	NoRayCaster(TCasterError),
	InvalidRay(Vec<InvalidRay>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct InvalidRay {
	entity: Entity,
	invalid_intersections: InvalidIntersections,
}

impl ErrorData for RayError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Ray Error"
	}

	fn into_details(self) -> impl std::fmt::Display {
		match self {
			RayError::NoRayCaster(error) => format!("No ray caster: {error:?}"),
			RayError::InvalidRay(beams) => format!("Invalid Rays: {beams:?}"),
		}
	}
}

pub(crate) struct Strategy<'a> {
	cast_for: CastRayFor,
	toi: TimeOfImpact,
	blocked_by: &'a HashSet<Blocker>,
	origin_offset: Units,
}

pub(crate) trait CastRayStrategy {
	type TQuery: QueryData;

	fn instance<'a>(&self, args: ROQueryItem<'a, 'a, Self::TQuery>) -> Option<Strategy<'a>>;
}

pub(crate) struct BeamStrategy;

impl CastRayStrategy for BeamStrategy {
	type TQuery = &'static Blockable;

	fn instance<'a>(
		&self,
		Blockable(obj): ROQueryItem<'a, 'a, Self::TQuery>,
	) -> Option<Strategy<'a>> {
		let PhysicalObject::Beam { range, blocked_by } = obj else {
			return None;
		};

		Some(Strategy {
			cast_for: CastRayFor::Beam,
			toi: TimeOfImpact::from(*range),
			blocked_by,
			origin_offset: Units::ZERO,
		})
	}
}

pub(crate) struct PreventTunnelingStrategy {
	delta_secs: f32,
}

impl CastRayStrategy for PreventTunnelingStrategy {
	type TQuery = (&'static Blockable, &'static Velocity, &'static Projectile);

	fn instance<'a>(
		&self,
		(Blockable(obj), velocity, Projectile { leading_edge }): ROQueryItem<'a, 'a, Self::TQuery>,
	) -> Option<Strategy<'a>> {
		let PhysicalObject::Fragile { destroyed_by } = obj else {
			return None;
		};

		Some(Strategy {
			cast_for: CastRayFor::TunnelingPrevention,
			toi: TimeOfImpact::from(Units::from(self.delta_secs * velocity.linear.length())),
			blocked_by: destroyed_by,
			origin_offset: *leading_edge,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{
			cast_rays::{CastRayFor, RayCastResult},
			collision_domains::Physical,
		},
		traits::ray_cast::{CastRayContinuouslySorted, InvalidIntersections, RayHit},
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::errors::Unreachable;
	use macros::simple_mock;
	use mockall::predicate::eq;
	use std::collections::{HashMap, HashSet};
	use testing::{Mock, SingleThreadedApp, assert_eq_approx, fake_entity};
	use zyheeda_core::collections::sorted::Sorted;

	struct _Strategy;

	impl CastRayStrategy for _Strategy {
		type TQuery = &'static _StrategyArgs;

		fn instance<'a>(
			&self,
			_StrategyArgs {
				cast_for,
				toi,
				blockers,
				offset,
			}: ROQueryItem<'a, 'a, Self::TQuery>,
		) -> Option<Strategy<'a>> {
			Some(Strategy {
				cast_for: *cast_for,
				toi: *toi,
				blocked_by: blockers,
				origin_offset: *offset,
			})
		}
	}

	#[derive(Component)]
	#[require(CastRays, GlobalTransform)]
	struct _StrategyArgs {
		cast_for: CastRayFor,
		toi: TimeOfImpact,
		blockers: HashSet<Blocker>,
		offset: Units,
	}

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

	fn setup(mut new_mock: impl FnMut(&mut World) -> Mock_RayCaster) -> App {
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
				_StrategyArgs {
					cast_for: CastRayFor::TunnelingPrevention,
					toi: toi!(11000.),
					blockers: HashSet::from([]),
					offset: Units::from(0.5),
				},
				Physical::Contact,
			));

			_ = app.world_mut().run_system_once_with(
				CastRays::execute_internal::<_Strategy, Res<_GetRayCaster>, Unreachable>,
				_Strategy,
			)?;

			fn assert_call_args(mock: &mut Mock_RayCaster) {
				mock.expect_cast_ray_continuously_sorted()
					.once()
					.with(eq(ray_args(Vec3::new(1., 2. - 0.5, 3.), Dir3::NEG_Y)))
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
					_StrategyArgs {
						cast_for: CastRayFor::TunnelingPrevention,
						toi: toi!(11000.),
						blockers: HashSet::from([]),
						offset: Units::from(1.),
					},
					Physical::Contact,
				))
				.id();

			_ = app.world_mut().run_system_once_with(
				CastRays::execute_internal::<_Strategy, Res<_GetRayCaster>, Unreachable>,
				_Strategy,
			)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::TunnelingPrevention,
						RayCastResult {
							args: ray_args(Vec3::new(0., 0., -1.), Dir3::NEG_Z),
							toi: toi!(11000.),
							hit: None,
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
			let hit = &mut None;
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn((
							BlockerTypes(HashSet::from([Blocker::Force, Blocker::Physical])),
							Physical::Contact,
						))
						.id();
					*hit = Some(blocker);
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
					_StrategyArgs {
						cast_for: CastRayFor::Beam,
						toi: toi!(11000.),
						blockers: HashSet::from([Blocker::Force, Blocker::Character]),
						offset: Units::ZERO,
					},
					Physical::Contact,
				))
				.id();

			_ = app.world_mut().run_system_once_with(
				CastRays::execute_internal::<_Strategy, Res<_GetRayCaster>, Unreachable>,
				_Strategy,
			)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(110.),
							hit: *hit,
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
			let hit = &mut None;
			let mut app = setup(|world| {
				Mock_RayCaster::new_mock(|mock| {
					let blocker = world
						.spawn((BlockerTypes(HashSet::from([
							Blocker::Force,
							Blocker::Physical,
						])),))
						.id();
					let collider = world.spawn((ColliderOf(blocker), Physical::Contact)).id();
					*hit = Some(collider);
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
					_StrategyArgs {
						cast_for: CastRayFor::Beam,
						toi: toi!(11000.),
						blockers: HashSet::from([Blocker::Force, Blocker::Character]),
						offset: Units::ZERO,
					},
					Physical::Contact,
				))
				.id();

			_ = app.world_mut().run_system_once_with(
				CastRays::execute_internal::<_Strategy, Res<_GetRayCaster>, Unreachable>,
				_Strategy,
			)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(110.),
							hit: *hit
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
					_StrategyArgs {
						cast_for: CastRayFor::Beam,
						toi: toi!(11000.),
						blockers: HashSet::from([Blocker::Force, Blocker::Character]),
						offset: Units::ZERO,
					},
					Physical::Contact,
				))
				.id();

			_ = app.world_mut().run_system_once_with(
				CastRays::execute_internal::<_Strategy, Res<_GetRayCaster>, Unreachable>,
				_Strategy,
			)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(11000.),
							hit: None
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
					_StrategyArgs {
						cast_for: CastRayFor::Beam,
						toi: toi!(11000.),
						blockers: HashSet::from([Blocker::Force, Blocker::Character]),
						offset: Units::ZERO,
					},
					Physical::Contact,
				))
				.id();

			_ = app.world_mut().run_system_once_with(
				CastRays::execute_internal::<_Strategy, Res<_GetRayCaster>, Unreachable>,
				_Strategy,
			)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(11000.),
							hit: None
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
					_StrategyArgs {
						cast_for: CastRayFor::Beam,
						toi: toi!(11000.),
						blockers: HashSet::from([Blocker::Force, Blocker::Character]),
						offset: Units::ZERO,
					},
					Physical::Contact,
				))
				.id();

			_ = app.world_mut().run_system_once_with(
				CastRays::execute_internal::<_Strategy, Res<_GetRayCaster>, Unreachable>,
				_Strategy,
			)?;

			assert_eq_approx!(
				Some(&CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							args: ray_args(Vec3::ZERO, Dir3::NEG_Z),
							toi: toi!(11000.),
							hit: None
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
				.spawn(_StrategyArgs {
					cast_for: CastRayFor::Beam,
					toi: toi!(11000.),
					blockers: HashSet::from([]),
					offset: Units::ZERO,
				})
				.id();

			let result = app.world_mut().run_system_once_with(
				CastRays::execute_internal::<_Strategy, Res<_GetRayCaster>, Unreachable>,
				_Strategy,
			)?;

			assert_eq!(
				Err(RayError::InvalidRay(vec![InvalidRay {
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
			app.world_mut().spawn(_StrategyArgs {
				cast_for: CastRayFor::Beam,
				toi: toi!(11000.),
				blockers: HashSet::from([]),
				offset: Units::ZERO,
			});

			let result = app.world_mut().run_system_once_with(
				CastRays::execute_internal::<_Strategy, Res<_GetRayCaster>, Unreachable>,
				_Strategy,
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
			app.world_mut().spawn(_StrategyArgs {
				cast_for: CastRayFor::Beam,
				toi: toi!(11000.),
				blockers: HashSet::from([]),
				offset: Units::ZERO,
			});

			let result = app.world_mut().run_system_once_with(
				CastRays::execute_internal::<_Strategy, _FaultyRayCaster, _CasterError>,
				_Strategy,
			)?;

			assert_eq!(Err(RayError::NoRayCaster(_CasterError)), result);
			Ok(())
		}
	}
}
