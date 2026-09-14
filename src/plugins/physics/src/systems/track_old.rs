use crate::components::impacted::Impacted;
use bevy::prelude::*;

impl Impacted {
	pub(crate) fn track_old(impacted: Query<&mut Self>) {
		for mut impacted in impacted {
			impacted.old_impact_points = impacted.impact_points.clone();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{traits::handles_physics::ImpactStrength, vec_not_nan};
	use std::collections::HashMap;
	use testing::SingleThreadedApp;
	use zyheeda_core::prelude::*;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Impacted::track_old);

		app
	}

	#[test]
	fn copy_current_to_old() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Impacted {
				impact_points: HashMap::from([(
					vec_not_nan!(1., 2., 3.),
					new_f32!(ImpactStrength(0.2)),
				)]),
				..default()
			})
			.id();

		app.update();

		assert_eq!(
			Some(&Impacted {
				impact_points: HashMap::from([(
					vec_not_nan!(1., 2., 3.),
					new_f32!(ImpactStrength(0.2)),
				)]),
				old_impact_points: HashMap::from([(
					vec_not_nan!(1., 2., 3.),
					new_f32!(ImpactStrength(0.2)),
				)]),
			}),
			app.world().entity(entity).get::<Impacted>()
		);
	}
}
