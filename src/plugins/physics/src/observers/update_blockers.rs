use crate::{components::blocker_types::BlockerTypes, traits::update_blockers::UpdateBlockers};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl<T> UpdateBlockersObserver for T where T: Component + UpdateBlockers {}

pub(crate) trait UpdateBlockersObserver: Component + Sized + UpdateBlockers {
	fn update_blockers(
		on_insert: On<Insert, Self>,
		mut commands: ZyheedaCommands,
		mut effects: Query<(&Self, Option<&mut BlockerTypes>)>,
	) {
		let entity = on_insert.entity;
		let Ok((effect, blockers)) = effects.get_mut(entity) else {
			return;
		};

		match blockers {
			Some(mut blockers) => {
				effect.update(&mut blockers);
			}
			None => {
				let mut blockers = BlockerTypes::default();
				effect.update(&mut blockers);
				commands.try_apply_on(&entity, |mut e| {
					e.try_insert(blockers);
				});
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_physics::physical_bodies::Blocker;

	#[derive(Component)]
	struct _Effect(Blocker);

	impl UpdateBlockers for _Effect {
		fn update(&self, BlockerTypes(blockers): &mut BlockerTypes) {
			blockers.insert(self.0);
		}
	}

	fn setup() -> App {
		let mut app = App::new();

		app.add_observer(_Effect::update_blockers);

		app
	}

	#[test]
	fn add_blocker() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn((_Effect(Blocker::Force), BlockerTypes::from([])));

		assert_eq!(
			Some(&BlockerTypes::from([Blocker::Force])),
			entity.get::<BlockerTypes>(),
		);
	}

	#[test]
	fn insert_blockers() {
		let mut app = setup();

		let entity = app.world_mut().spawn(_Effect(Blocker::Force));

		assert_eq!(
			Some(&BlockerTypes::from([Blocker::Force])),
			entity.get::<BlockerTypes>(),
		);
	}

	#[test]
	fn add_blocker_when_reinserted() {
		let mut app = setup();

		let mut entity = app
			.world_mut()
			.spawn((_Effect(Blocker::Force), BlockerTypes::from([])));
		entity.insert(_Effect(Blocker::Physical));

		assert_eq!(
			Some(&BlockerTypes::from([Blocker::Force, Blocker::Physical])),
			entity.get::<BlockerTypes>(),
		);
	}
}
