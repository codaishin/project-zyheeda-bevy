use crate::traits::update_blockers::UpdateBlockers;
use bevy::prelude::*;
use common::{components::is_blocker::IsBlocker, traits::try_insert_on::TryInsertOn};

impl<T> UpdateBlockersObserver for T where T: Component + UpdateBlockers {}

pub(crate) trait UpdateBlockersObserver: Component + Sized + UpdateBlockers {
	fn update_blockers_observer(
		trigger: Trigger<OnInsert, Self>,
		mut commands: Commands,
		mut effects: Query<(&Self, Option<&mut IsBlocker>)>,
	) {
		let entity = trigger.target();
		let Ok((effect, blockers)) = effects.get_mut(entity) else {
			return;
		};

		match blockers {
			Some(mut blockers) => {
				effect.update_blockers(&mut blockers);
			}
			None => {
				let mut blockers = IsBlocker::from([]);
				effect.update_blockers(&mut blockers);
				commands.try_insert_on(entity, blockers);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::components::is_blocker::Blocker;

	#[derive(Component)]
	struct _Effect(Blocker);

	impl UpdateBlockers for _Effect {
		fn update_blockers(&self, IsBlocker(blockers): &mut IsBlocker) {
			blockers.insert(self.0);
		}
	}

	fn setup() -> App {
		let mut app = App::new();

		app.add_observer(_Effect::update_blockers_observer);

		app
	}

	#[test]
	fn add_blocker() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn((_Effect(Blocker::Force), IsBlocker::from([])));

		assert_eq!(
			Some(&IsBlocker::from([Blocker::Force])),
			entity.get::<IsBlocker>(),
		);
	}

	#[test]
	fn insert_blockers() {
		let mut app = setup();

		let entity = app.world_mut().spawn(_Effect(Blocker::Force));

		assert_eq!(
			Some(&IsBlocker::from([Blocker::Force])),
			entity.get::<IsBlocker>(),
		);
	}

	#[test]
	fn add_blocker_when_reinserted() {
		let mut app = setup();

		let mut entity = app
			.world_mut()
			.spawn((_Effect(Blocker::Force), IsBlocker::from([])));
		entity.insert(_Effect(Blocker::Physical));

		assert_eq!(
			Some(&IsBlocker::from([Blocker::Force, Blocker::Physical])),
			entity.get::<IsBlocker>(),
		);
	}
}
