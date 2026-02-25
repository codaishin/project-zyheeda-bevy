use crate::{
	components::{RayCasterArgs, RayFilter, prevent_tunneling::PreventTunneling},
	system_params::update_ongoing_interactions::UpdateOngoingInteractions,
	traits::{
		cast_ray::{CastRayContinuouslySorted, GetContinuousSortedRayCaster, InvalidIntersections},
		send_collision_interaction::PushOngoingInteraction,
	},
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::{
	errors::{ErrorData, Level},
	traits::handles_physics::TimeOfImpact,
};
use std::time::Duration;

impl PreventTunneling {
	pub(crate) fn system(
		delta: In<Duration>,
		cast_ray: StaticSystemParam<ReadRapierContext>,
		interactions: StaticSystemParam<UpdateOngoingInteractions>,
		colliders: Query<(Entity, &Self, &Velocity, &GlobalTransform)>,
	) -> Result<(), TunnelingRayError> {
		Self::system_internal(delta, cast_ray, interactions, colliders)
	}

	fn system_internal<TGetRayCaster, TCasterError, TInteractions>(
		In(delta): In<Duration>,
		cast_ray: StaticSystemParam<TGetRayCaster>,
		mut interactions: StaticSystemParam<TInteractions>,
		colliders: Query<(Entity, &Self, &Velocity, &GlobalTransform)>,
	) -> Result<(), TunnelingRayError<TCasterError>>
	where
		TGetRayCaster: for<'w, 's> SystemParam<
			Item<'w, 's>: GetContinuousSortedRayCaster<RayCasterArgs, TError = TCasterError>,
		>,
		TInteractions: for<'w, 's> SystemParam<Item<'w, 's>: PushOngoingInteraction>,
	{
		let cast_ray = match cast_ray.get_continuous_sorted_ray_caster() {
			Ok(cast_ray) => cast_ray,
			Err(error) => return Err(TunnelingRayError::NoRayCaster(error)),
		};
		let delta_secs = delta.as_secs_f32();
		let mut invalid_rays = vec![];

		for (entity, PreventTunneling { leading_edge }, velocity, transform) in colliders {
			let max_toi = velocity.linvel.length() * delta_secs;
			let Ok(max_toi) = TimeOfImpact::try_from_f32(max_toi) else {
				invalid_rays.push(InvalidRay {
					entity,
					invalid_intersections: InvalidIntersections(vec![]),
					invalid_forward: Some(InvalidForward {
						delta_secs,
						velocity: *velocity,
					}),
				});
				continue;
			};
			let Ok(direction) = Dir3::try_from(velocity.linvel) else {
				continue;
			};
			let origin = transform.translation() + direction * **leading_edge;
			let ray = RayCasterArgs {
				max_toi,
				direction,
				origin,
				solid: true,
				filter: RayFilter::default().exclude_rigid_body(entity),
			};
			let hits = match cast_ray.cast_ray_continuously_sorted(&ray) {
				Ok(hits) => hits,
				Err(invalid_intersections) => {
					invalid_rays.push(InvalidRay {
						entity,
						invalid_intersections,
						invalid_forward: None,
					});
					continue;
				}
			};

			for hit in hits {
				interactions.push_ongoing_interaction(hit.entity, entity);
				interactions.push_ongoing_interaction(entity, hit.entity);
			}
		}

		if !invalid_rays.is_empty() {
			return Err(TunnelingRayError::InvalidRays(invalid_rays));
		}

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum TunnelingRayError<TCasterError = BevyError> {
	NoRayCaster(TCasterError),
	InvalidRays(Vec<InvalidRay>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct InvalidRay {
	entity: Entity,
	invalid_intersections: InvalidIntersections,
	invalid_forward: Option<InvalidForward>,
}

impl ErrorData for TunnelingRayError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Tunneling Ray Error"
	}

	fn into_details(self) -> impl std::fmt::Display {
		match self {
			TunnelingRayError::NoRayCaster(error) => format!("No ray caster: {error:?}"),
			TunnelingRayError::InvalidRays(rays) => format!("Invalid rays: {rays:?}"),
		}
	}
}

#[derive(Debug)]
#[cfg_attr(not(test), derive(PartialEq))]
pub(crate) struct InvalidForward {
	delta_secs: f32,
	velocity: Velocity,
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::traits::cast_ray::{InvalidIntersections, RayHit};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{tools::Units, traits::handles_physics::TimeOfImpact};
	use core::f32;
	use macros::{NestedMocks, simple_mock};
	use mockall::{automock, predicate::eq};
	use testing::{Mock, NestedMocks, SingleThreadedApp, assert_eq_approx, fake_entity};
	use zyheeda_core::prelude::Sorted;

	// Implement equality for `NaN` values for testing only
	impl PartialEq for InvalidForward {
		fn eq(&self, other: &Self) -> bool {
			let deltas_match = self.delta_secs == other.delta_secs
				|| self.delta_secs.is_nan() && other.delta_secs.is_nan();
			let velocities_match = self.velocity == other.velocity
				|| self.velocity.linvel.is_nan() && other.velocity.linvel.is_nan();

			deltas_match && velocities_match
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Error;

	#[derive(Resource)]
	struct _GetRayCaster {
		mock: Option<Mock_RayCaster>,
	}

	impl GetContinuousSortedRayCaster<RayCasterArgs> for Res<'_, _GetRayCaster> {
		type TError = _Error;

		type TRayCaster<'a>
			= &'a Mock_RayCaster
		where
			Self: 'a;

		fn get_continuous_sorted_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError> {
			match self.mock.as_ref() {
				Some(mock) => Ok(mock),
				None => Err(_Error),
			}
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

	#[derive(Resource, NestedMocks)]
	struct _OngoingCollisions {
		mock: Mock_OngoingCollisions,
	}

	impl Default for _OngoingCollisions {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_push_ongoing_interaction().return_const(());
			})
		}
	}

	impl PushOngoingInteraction for ResMut<'_, _OngoingCollisions> {
		fn push_ongoing_interaction(&mut self, a: Entity, b: Entity) {
			self.mock.push_ongoing_interaction(a, b);
		}
	}

	#[automock]
	impl PushOngoingInteraction for _OngoingCollisions {
		fn push_ongoing_interaction(&mut self, a: Entity, b: Entity) {
			self.mock.push_ongoing_interaction(a, b);
		}
	}

	fn setup(ray_caster: Option<Mock_RayCaster>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_OngoingCollisions>();
		app.insert_resource(_GetRayCaster { mock: ray_caster });

		app
	}

	#[test]
	fn push_interactions() -> Result<(), RunSystemError> {
		let mut app = setup(Some(Mock_RayCaster::new_mock(|mock| {
			let mut hits = Sorted::default();
			hits.push(RayHit {
				entity: fake_entity!(123),
				toi: TimeOfImpact::from(Units::from_u8(42)),
			});
			mock.expect_cast_ray_continuously_sorted()
				.return_const(Ok(hits));
		})));
		let entity = app
			.world_mut()
			.spawn((
				PreventTunneling {
					leading_edge: Units::from_u8(1),
				},
				Velocity::linear(Vec3::X),
			))
			.id();
		app.insert_resource(_OngoingCollisions::new().with_mock(|mock| {
			mock.expect_push_ongoing_interaction()
				.times(1)
				.with(eq(entity), eq(fake_entity!(123)))
				.return_const(());
			mock.expect_push_ongoing_interaction()
				.times(1)
				.with(eq(fake_entity!(123)), eq(entity))
				.return_const(());
		}));

		_ = app.world_mut().run_system_once_with(
			PreventTunneling::system_internal::<
				Res<_GetRayCaster>,
				_Error,
				ResMut<_OngoingCollisions>,
			>,
			Duration::from_secs(1),
		)?;

		Ok(())
	}

	#[test]
	fn cast_with_proper_ray() -> Result<(), RunSystemError> {
		let mut app = setup(None);
		let entity = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 2., 3.),
				Velocity::linear(Vec3::new(4., 5., 6.)),
				PreventTunneling {
					leading_edge: Units::from_u8(1),
				},
			))
			.id();
		app.world_mut().insert_resource(_GetRayCaster {
			mock: Some(Mock_RayCaster::new_mock(|mock| {
				mock.expect_cast_ray_continuously_sorted()
					.times(1)
					.withf(move |ray| {
						assert_eq_approx!(
							RayCasterArgs {
								origin: Vec3::new(1., 2., 3.) + Vec3::new(4., 5., 6.).normalize(),
								direction: Dir3::try_from(Vec3::new(4., 5., 6.).normalize())
									.unwrap(),
								filter: RayFilter::default().exclude_rigid_body(entity),
								solid: true,
								max_toi: TimeOfImpact::from(Units::from(
									Vec3::new(4., 5., 6.).length() * 0.5
								)),
							},
							ray,
							0.001,
						);

						true
					})
					.return_const(Ok(Sorted::default()));
			})),
		});

		_ = app.world_mut().run_system_once_with(
			PreventTunneling::system_internal::<
				Res<_GetRayCaster>,
				_Error,
				ResMut<_OngoingCollisions>,
			>,
			Duration::from_millis(500),
		)?;

		Ok(())
	}

	#[test]
	fn return_no_ray_caster() -> Result<(), RunSystemError> {
		let mut app = setup(None);
		app.world_mut().spawn(PreventTunneling {
			leading_edge: Units::from_u8(1),
		});

		let result = app.world_mut().run_system_once_with(
			PreventTunneling::system_internal::<
				Res<_GetRayCaster>,
				_Error,
				ResMut<_OngoingCollisions>,
			>,
			Duration::from_secs(1),
		)?;

		assert_eq!(Err(TunnelingRayError::NoRayCaster(_Error)), result);
		Ok(())
	}

	#[test]
	fn return_invalid_velocity() -> Result<(), RunSystemError> {
		let mut app = setup(Some(Mock_RayCaster::new_mock(|mock| {
			mock.expect_cast_ray_continuously_sorted()
				.return_const(Ok(Sorted::default()));
		})));
		let entity = app
			.world_mut()
			.spawn((
				PreventTunneling {
					leading_edge: Units::from_u8(1),
				},
				Velocity::linear(Vec3::new(f32::NAN, 2., 3.)),
			))
			.id();

		let result = app.world_mut().run_system_once_with(
			PreventTunneling::system_internal::<
				Res<_GetRayCaster>,
				_Error,
				ResMut<_OngoingCollisions>,
			>,
			Duration::from_secs(2),
		)?;

		assert_eq!(
			Err(TunnelingRayError::InvalidRays(vec![InvalidRay {
				entity,
				invalid_intersections: InvalidIntersections(vec![]),
				invalid_forward: Some(InvalidForward {
					delta_secs: 2.,
					velocity: Velocity::linear(Vec3::new(f32::NAN, 2., 3.))
				})
			}])),
			result
		);
		Ok(())
	}

	#[test]
	fn return_invalid_hits() -> Result<(), RunSystemError> {
		let mut app = setup(Some(Mock_RayCaster::new_mock(|mock| {
			mock.expect_cast_ray_continuously_sorted()
				.return_const(Err(InvalidIntersections(vec![Vec3::new(1., 2., 3.)])));
		})));
		let entity = app
			.world_mut()
			.spawn((
				PreventTunneling {
					leading_edge: Units::from_u8(1),
				},
				Velocity::linear(Vec3::X),
			))
			.id();

		let result = app.world_mut().run_system_once_with(
			PreventTunneling::system_internal::<
				Res<_GetRayCaster>,
				_Error,
				ResMut<_OngoingCollisions>,
			>,
			Duration::from_secs(1),
		)?;

		assert_eq!(
			Err(TunnelingRayError::InvalidRays(vec![InvalidRay {
				entity,
				invalid_intersections: InvalidIntersections(vec![Vec3::new(1., 2., 3.)]),
				invalid_forward: None
			}])),
			result
		);
		Ok(())
	}

	#[test]
	fn return_ok() -> Result<(), RunSystemError> {
		let mut app = setup(Some(Mock_RayCaster::new_mock(|mock| {
			mock.expect_cast_ray_continuously_sorted()
				.return_const(Ok(Sorted::default()));
		})));
		app.world_mut().spawn((
			PreventTunneling {
				leading_edge: Units::from_u8(1),
			},
			Velocity::linear(Vec3::X),
		));

		let result = app.world_mut().run_system_once_with(
			PreventTunneling::system_internal::<
				Res<_GetRayCaster>,
				_Error,
				ResMut<_OngoingCollisions>,
			>,
			Duration::from_secs(1),
		)?;

		assert_eq!(Ok(()), result);
		Ok(())
	}
}
