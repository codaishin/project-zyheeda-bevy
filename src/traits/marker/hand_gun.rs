use super::GetMarkerMeta;
use crate::{
	components::{Marker, Side, SlotKey},
	errors::Error,
	markers::{meta::MarkerMeta, HandGun, Left, Right},
};
use bevy::ecs::system::EntityCommands;

fn slot_error(slot: SlotKey) -> Error {
	Error(format!("{:?} is not a valid handgun slot", slot))
}

fn add_hand_gun(entity: &mut EntityCommands, slot: SlotKey) -> Result<(), Error> {
	let SlotKey::Hand(side) = slot else {
		return Err(slot_error(slot));
	};

	match side {
		Side::Right => entity.insert(Marker::<(HandGun, Right)>::new()),
		Side::Left => entity.insert(Marker::<(HandGun, Left)>::new()),
	};

	Ok(())
}

fn remove_hand_gun(entity: &mut EntityCommands, slot: SlotKey) -> Result<(), Error> {
	let SlotKey::Hand(side) = slot else {
		return Err(slot_error(slot));
	};

	match side {
		Side::Right => entity.remove::<Marker<(HandGun, Right)>>(),
		Side::Left => entity.remove::<Marker<(HandGun, Left)>>(),
	};

	Ok(())
}

impl GetMarkerMeta for HandGun {
	fn marker() -> MarkerMeta {
		MarkerMeta {
			insert_fn: add_hand_gun,
			remove_fn: remove_hand_gun,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Side, SlotKey},
		markers::{Left, Right},
		traits::marker::test_tools::{fake_log, insert_lazy, remove_lazy, FakeLog},
	};
	use bevy::{
		app::{App, Update},
		ecs::system::IntoSystem,
	};

	#[test]
	fn add_markers_right() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let marker = HandGun::marker();
		let slot = SlotKey::Hand(Side::Right);

		app.add_systems(
			Update,
			insert_lazy(marker, agent, slot).pipe(fake_log(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(true, false),
			(
				agent.contains::<Marker<(HandGun, Right)>>(),
				agent.contains::<FakeLog>()
			)
		);
	}

	#[test]
	fn add_markers_left() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let marker = HandGun::marker();
		let slot = SlotKey::Hand(Side::Left);

		app.add_systems(
			Update,
			insert_lazy(marker, agent, slot).pipe(fake_log(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(true, false),
			(
				agent.contains::<Marker<(HandGun, Left)>>(),
				agent.contains::<FakeLog>()
			)
		);
	}

	#[test]
	fn add_markers_no_hand() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let marker = HandGun::marker();
		let slot = SlotKey::Legs;

		app.add_systems(
			Update,
			insert_lazy(marker, agent, slot).pipe(fake_log(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false, Some(slot_error(slot))),
			(
				agent.contains::<Marker<(HandGun, Left)>>(),
				agent.contains::<Marker<(HandGun, Right)>>(),
				agent.get::<FakeLog>().map(|l| l.error.clone()),
			)
		);
	}

	#[test]
	fn remove_markers_right() {
		let mut app = App::new();
		let agent = app.world.spawn(Marker::<(HandGun, Right)>::new()).id();
		let marker = HandGun::marker();
		let slot = SlotKey::Hand(Side::Right);

		app.add_systems(
			Update,
			remove_lazy(marker, agent, slot).pipe(fake_log(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(
				agent.contains::<Marker<(HandGun, Right)>>(),
				agent.contains::<FakeLog>()
			)
		);
	}

	#[test]
	fn remove_markers_left() {
		let mut app = App::new();
		let agent = app.world.spawn(Marker::<(HandGun, Left)>::new()).id();
		let marker = HandGun::marker();
		let slot = SlotKey::Hand(Side::Left);

		app.add_systems(
			Update,
			remove_lazy(marker, agent, slot).pipe(fake_log(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(
				agent.contains::<Marker<(HandGun, Left)>>(),
				agent.contains::<FakeLog>()
			)
		);
	}

	#[test]
	fn remove_markers_no_hand() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let marker = HandGun::marker();
		let slot = SlotKey::Legs;

		app.add_systems(
			Update,
			remove_lazy(marker, agent, slot).pipe(fake_log(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(slot_error(slot)),
			agent.get::<FakeLog>().map(|l| l.error.clone())
		);
	}
}
