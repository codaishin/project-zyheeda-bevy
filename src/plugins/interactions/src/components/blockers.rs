use bevy::prelude::*;
use common::blocker::{Blocker, BlockerInsertCommand};
use std::collections::HashSet;

#[derive(Component, Default, Debug, PartialEq)]
pub(crate) struct Blockers(pub(crate) HashSet<Blocker>);

impl Blockers {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(blockers: [Blocker; N]) -> Self {
		Self(HashSet::from(blockers))
	}
}

impl ApplyBlockerInsertion for BlockerInsertCommand {}

pub(crate) trait ApplyBlockerInsertion {
	fn apply(
		mut commands: Commands,
		agents: Query<(Entity, &BlockerInsertCommand)>,
		mut blockers: Query<&mut Blockers>,
	) {
		for (entity, BlockerInsertCommand(values)) in &agents {
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.remove::<BlockerInsertCommand>();

			if let Ok(mut blockers) = blockers.get_mut(entity.id()) {
				blockers.0.extend(values);
			} else {
				entity.try_insert(Blockers(values.clone()));
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, BlockerInsertCommand::apply);

		app
	}

	#[test]
	fn insert_physical_blocker() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Blocker::insert([Blocker::Physical]))
			.id();

		app.update();

		assert_eq!(
			Some(&Blockers::new([Blocker::Physical])),
			app.world().entity(entity).get::<Blockers>()
		);
	}

	#[test]
	fn insert_force_blocker() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Blocker::insert([Blocker::Force]))
			.id();

		app.update();

		assert_eq!(
			Some(&Blockers::new([Blocker::Force])),
			app.world().entity(entity).get::<Blockers>()
		);
	}

	#[test]
	fn insert_force_when_physical_present_blocker() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Blockers::new([Blocker::Force]),
				Blocker::insert([Blocker::Physical]),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Blockers::new([Blocker::Force, Blocker::Physical])),
			app.world().entity(entity).get::<Blockers>()
		);
	}

	#[test]
	fn remove_insert_command() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((Blockers::new([]), Blocker::insert([])))
			.id();

		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<BlockerInsertCommand>()
		);
	}
}
