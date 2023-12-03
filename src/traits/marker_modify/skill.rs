use super::MarkerModify;
use crate::{
	components::{Active, Skill},
	errors::Error,
};
use bevy::ecs::system::EntityCommands;

impl MarkerModify for Skill<Active> {
	fn insert_markers(&self, agent: &mut EntityCommands) -> Result<(), Error> {
		(self.marker.insert_fn)(agent, self.data.slot)
	}

	fn remove_markers(&self, agent: &mut EntityCommands) -> Result<(), Error> {
		(self.marker.remove_fn)(agent, self.data.slot)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Side, SlotKey},
		errors::Level,
		markers::meta::MarkerMeta,
		traits::marker_modify::test_tools::{insert_system, remove_system, FakeResult},
	};
	use bevy::{
		app::{App, Update},
		ecs::component::Component,
		prelude::default,
	};

	#[derive(Component)]
	struct MockMarker {
		pub slot: SlotKey,
	}

	#[test]
	fn insert_markers() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let slot = SlotKey::Hand(Side::Right);
		let skill = Skill {
			data: Active { slot, ..default() },
			marker: MarkerMeta {
				insert_fn: |agent, slot| {
					agent.insert(MockMarker { slot });
					Ok(())
				},
				..default()
			},
			..default()
		};

		app.add_systems(Update, insert_system(agent, skill));
		app.update();

		let marker = app.world.entity(agent).get::<MockMarker>();

		assert_eq!(Some(slot), marker.map(|m| m.slot));
	}

	#[test]
	fn insert_markers_result() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let slot = SlotKey::Hand(Side::Right);
		let skill = Skill {
			data: Active { slot, ..default() },
			marker: MarkerMeta {
				insert_fn: |_, _| {
					Err(Error {
						msg: "ERROR".to_owned(),
						lvl: Level::Warning,
					})
				},
				..default()
			},
			..default()
		};

		app.add_systems(Update, insert_system(agent, skill));
		app.update();

		let result = app.world.entity(agent).get::<FakeResult>().unwrap();

		assert_eq!(
			Err(Error {
				msg: "ERROR".to_owned(),
				lvl: Level::Warning,
			}),
			result.result
		);
	}

	#[test]
	fn remove_markers() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let slot = SlotKey::Hand(Side::Left);
		let skill = Skill {
			data: Active { slot, ..default() },
			marker: MarkerMeta {
				remove_fn: |agent, slot| {
					agent.insert(MockMarker { slot });
					Ok(())
				},
				..default()
			},
			..default()
		};

		app.add_systems(Update, remove_system(agent, skill));
		app.update();

		let marker = app.world.entity(agent).get::<MockMarker>();

		assert_eq!(Some(slot), marker.map(|m| m.slot));
	}

	#[test]
	fn remove_markers_result() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let slot = SlotKey::Hand(Side::Right);
		let skill = Skill {
			data: Active { slot, ..default() },
			marker: MarkerMeta {
				remove_fn: |_, _| {
					Err(Error {
						msg: "ERROR".to_owned(),
						lvl: Level::Warning,
					})
				},
				..default()
			},
			..default()
		};

		app.add_systems(Update, remove_system(agent, skill));
		app.update();

		let result = app.world.entity(agent).get::<FakeResult>().unwrap();

		assert_eq!(
			Err(Error {
				msg: "ERROR".to_owned(),
				lvl: Level::Warning,
			}),
			result.result
		);
	}
}
