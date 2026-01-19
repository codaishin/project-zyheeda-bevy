use crate::resources::ongoing_interactions::OngoingInteractions;
use bevy::prelude::*;

impl OngoingInteractions {
	pub(crate) fn clear(mut interactions: ResMut<Self>) {
		interactions.targets.clear();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::{HashMap, HashSet};
	use testing::SingleThreadedApp;

	fn setup(interactions: OngoingInteractions) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(interactions);
		app.add_systems(Update, OngoingInteractions::clear);

		app
	}

	#[test]
	fn clear() {
		let mut app = setup(OngoingInteractions {
			targets: HashMap::from([(Entity::from_raw(42), HashSet::from([]))]),
		});

		app.update();

		assert_eq!(
			&OngoingInteractions::default(),
			app.world().resource::<OngoingInteractions>(),
		);
	}
}
