use crate::system_params::impacted::ImpactedContext;
use common::prelude::*;
use zyheeda_core::prelude::*;

impl<'c> Iterate<'c> for ImpactedContext<'c> {
	type TItem = Impact;
	type TIter = Iter<'c>;

	fn iterate(&'c self) -> Self::TIter {
		Iter {
			it: self.impacted.impact_points.iter(),
		}
	}
}

pub struct Iter<'c> {
	it: std::collections::hash_map::Iter<'c, VecNotNan<3>, ImpactStrength>,
}

impl Iterator for Iter<'_> {
	type Item = Impact;

	fn next(&mut self) -> Option<Self::Item> {
		let (position, strength) = self.it.next()?;

		Some(Impact {
			position: GlobalVec3(*position),
			strength: *strength,
		})
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{components::impacted::Impacted, system_params::impacted::ImpactedParam};
	use bevy::{
		app::App,
		ecs::system::{RunSystemError, RunSystemOnce, StaticSystemParam},
		prelude::*,
	};
	use common::traits::handles_physics::Impacted as ImpactedKey;
	use std::collections::{HashMap, HashSet};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn iter_impacts() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Impacted {
				impact_points: HashMap::from([
					(vec_not_nan!(1., 2., 3.), new_f32!(ImpactStrength(1.))),
					(vec_not_nan!(3., 4., 5.), new_f32!(ImpactStrength(0.5))),
				]),
			})
			.id();

		let impacts =
			app.world_mut()
				.run_system_once(move |i: StaticSystemParam<ImpactedParam>| {
					let ctx = ImpactedParam::try_get_context(&i, ImpactedKey { entity }).unwrap();
					ctx.iterate().collect::<HashSet<_>>()
				})?;

		assert_eq!(
			HashSet::from([
				Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(1.))
				},
				Impact {
					position: GlobalVec3(vec_not_nan!(3., 4., 5.)),
					strength: new_f32!(ImpactStrength(0.5))
				}
			]),
			impacts
		);
		Ok(())
	}
}
