use crate::{
	components::{Marker, Side, SlotKey},
	errors::{Error, Level},
};
use bevy::ecs::system::EntityCommands;

pub fn insert_hand_marker_fn<TOff: Send + Sync + 'static, TMain: Send + Sync + 'static>(
	entity: &mut EntityCommands,
	slot: SlotKey,
) -> Result<(), Error> {
	let SlotKey::Hand(side) = slot else {
		return Err(slot_error(slot));
	};

	match side {
		Side::Off => entity.insert(Marker::<TOff>::new()),
		Side::Main => entity.insert(Marker::<TMain>::new()),
	};

	Ok(())
}

fn slot_error(slot: SlotKey) -> Error {
	Error {
		msg: format!("{:?} is not a valid hand slot", slot),
		lvl: Level::Error,
	}
}

pub fn remove_hand_marker_fn<TOff: Send + Sync + 'static, TMain: Send + Sync + 'static>(
	entity: &mut EntityCommands,
	slot: SlotKey,
) -> Result<(), Error> {
	let SlotKey::Hand(side) = slot else {
		return Err(slot_error(slot));
	};

	match side {
		Side::Off => entity.remove::<Marker<TOff>>(),
		Side::Main => entity.remove::<Marker<TMain>>(),
	};

	Ok(())
}

#[cfg(test)]
mod insert_hand_tests {
	use super::*;
	use crate::{
		markers::test_tools::system,
		systems::log::tests::{fake_log_error_lazy, FakeErrorLog},
	};
	use bevy::{
		app::{App, Update},
		ecs::system::IntoSystem,
		prelude::Entity,
	};

	#[derive(Debug, PartialEq)]
	struct _Left;

	#[derive(Debug, PartialEq)]
	struct _Right;

	fn setup(slot: SlotKey) -> (App, Entity) {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let insert_fn = insert_hand_marker_fn::<_Left, _Right>;
		let log_fn = fake_log_error_lazy(agent);

		app.add_systems(Update, system(insert_fn, agent, slot).pipe(log_fn));

		(app, agent)
	}

	#[test]
	fn insert_off_hand() {
		let (mut app, agent) = setup(SlotKey::Hand(Side::Off));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Marker<_Left>>());
	}

	#[test]
	fn insert_main_hand() {
		let (mut app, agent) = setup(SlotKey::Hand(Side::Main));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Marker<_Right>>());
	}

	#[test]
	fn insert_error_when_not_hand_slot() {
		let (mut app, agent) = setup(SlotKey::Legs);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&FakeErrorLog(slot_error(SlotKey::Legs))),
			agent.get::<FakeErrorLog>(),
		);
	}
}

#[cfg(test)]
mod remove_hand_tests {
	use super::*;
	use crate::{
		markers::test_tools::system,
		systems::log::tests::{fake_log_error_lazy, FakeErrorLog},
	};
	use bevy::{
		app::{App, Update},
		ecs::system::IntoSystem,
		prelude::Entity,
	};

	#[derive(Debug, PartialEq)]
	struct _Left;

	#[derive(Debug, PartialEq)]
	struct _Right;

	fn setup_and_add_marker<TMarkerValue: Sync + Send + 'static>(slot: SlotKey) -> (App, Entity) {
		let mut app = App::new();
		let agent = app.world.spawn(Marker::<TMarkerValue>::new()).id();
		let remove_fn = remove_hand_marker_fn::<_Left, _Right>;
		let log_fn = fake_log_error_lazy(agent);

		app.add_systems(Update, system(remove_fn, agent, slot).pipe(log_fn));

		(app, agent)
	}

	#[test]
	fn remove_off_hand() {
		let (mut app, agent) = setup_and_add_marker::<_Left>(SlotKey::Hand(Side::Off));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Marker<_Left>>());
	}

	#[test]
	fn remove_main_hand() {
		let (mut app, agent) = setup_and_add_marker::<_Right>(SlotKey::Hand(Side::Main));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Marker<_Right>>());
	}

	#[test]
	fn remove_error_when_no_hand_slit() {
		let (mut app, agent) = setup_and_add_marker::<()>(SlotKey::Legs);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&FakeErrorLog(slot_error(SlotKey::Legs))),
			agent.get::<FakeErrorLog>()
		);
	}
}
