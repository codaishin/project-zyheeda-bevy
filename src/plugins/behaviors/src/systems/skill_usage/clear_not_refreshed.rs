use crate::components::skill_usage::SkillUsage;
use bevy::prelude::*;
use std::ops::DerefMut;

impl SkillUsage {
	pub(crate) fn clear_not_refreshed(mut skill_usages: Query<&mut Self>) {
		for mut skill_usage in &mut skill_usages {
			let Self {
				started_holding,
				holding,
				refreshed,
			} = skill_usage.deref_mut();

			started_holding.retain(|s| refreshed.contains(s));
			holding.retain(|s| refreshed.contains(s));
			refreshed.clear();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::slot::SlotKey;
	use std::collections::HashSet;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, SkillUsage::clear_not_refreshed);

		app
	}

	#[test]
	fn clear_not_refreshed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(SkillUsage {
				started_holding: HashSet::from([SlotKey(1), SlotKey(2)]),
				holding: HashSet::from([SlotKey(1), SlotKey(2), SlotKey(3)]),
				refreshed: HashSet::from([SlotKey(2)]),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&SkillUsage {
				started_holding: HashSet::from([SlotKey(2)]),
				holding: HashSet::from([SlotKey(2)]),
				refreshed: HashSet::from([]),
			}),
			app.world().entity(entity).get::<SkillUsage>(),
		);
	}
}
