use crate::components::cast_rays::CastRays;
use bevy::prelude::*;

impl CastRays {
	pub(crate) fn clear(cast_rays: Query<&mut Self>) {
		for mut cast_rays in cast_rays {
			cast_rays.results.clear();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::cast_rays::{CastRayFor, RayCastResult};
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, CastRays::clear);

		app
	}

	#[test]
	fn clear() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(CastRays {
				results: HashMap::from([(CastRayFor::Beam, RayCastResult::default())]),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&CastRays {
				results: HashMap::default()
			}),
			app.world().entity(entity).get::<CastRays>()
		);
	}
}
