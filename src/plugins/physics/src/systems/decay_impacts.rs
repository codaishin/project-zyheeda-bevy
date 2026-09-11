use crate::components::impacted::Impacted;
use bevy::prelude::*;
use common::prelude::*;
use std::time::Duration;

impl Impacted {
	pub(crate) fn decay_impacts(
		DecayPerSecond(decay_per_second): DecayPerSecond,
	) -> impl IntoSystem<In<Duration>, (), ()> {
		IntoSystem::into_system(
			move |In(duration): In<Duration>, impacts: Query<&mut Impacted>| {
				let delta_secs = duration.as_secs_f32();

				for mut impact in impacts {
					impact.impact_points.retain(|_, strength| {
						let decayed = **strength - *decay_per_second * delta_secs;

						let Ok(decayed) = ImpactStrength::try_from(decayed) else {
							return false;
						};

						*strength = decayed;
						true
					});
				}
			},
		)
	}
}

pub(crate) struct DecayPerSecond(pub(crate) ImpactStrength);

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::vec_not_nan;
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, assert_eq_approx};
	use zyheeda_core::prelude::*;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn decay_impacts() -> Result<(), RunSystemError> {
		let mut app = setup();
		let decay = DecayPerSecond(new_f32!(ImpactStrength(1.0)));
		let entity = app
			.world_mut()
			.spawn(Impacted {
				impact_points: HashMap::from([(
					vec_not_nan!(1., 2., 3.),
					new_f32!(ImpactStrength(1.0)),
				)]),
			})
			.id();

		app.world_mut()
			.run_system_once_with(Impacted::decay_impacts(decay), Duration::from_millis(100))?;

		assert_eq_approx!(
			Some(&Impacted {
				impact_points: HashMap::from([(
					vec_not_nan!(1., 2., 3.),
					new_f32!(ImpactStrength(0.9)),
				)]),
			}),
			app.world().entity(entity).get::<Impacted>(),
			0.01,
		);
		Ok(())
	}

	#[test]
	fn remove_fully_decayed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let decay = DecayPerSecond(new_f32!(ImpactStrength(1.0)));
		let entity = app
			.world_mut()
			.spawn(Impacted {
				impact_points: HashMap::from([(
					vec_not_nan!(1., 2., 3.),
					new_f32!(ImpactStrength(1.0)),
				)]),
			})
			.id();

		app.world_mut()
			.run_system_once_with(Impacted::decay_impacts(decay), Duration::from_secs(2))?;

		assert_eq!(
			Some(&Impacted {
				impact_points: HashMap::from([]),
			}),
			app.world().entity(entity).get::<Impacted>(),
		);
		Ok(())
	}
}
