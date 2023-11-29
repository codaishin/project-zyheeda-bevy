use super::GetMarkerMeta;
use crate::{
	components::{Active, Marker, Queued, Side, Skill, SlotKey},
	errors::{Error, Level},
	markers::{meta::MarkerMeta, HandGun, Left, Right},
};
use bevy::ecs::system::EntityCommands;

impl GetMarkerMeta for HandGun {
	fn marker() -> MarkerMeta {
		HANDGUN_META
	}
}

fn slot_error(slot: SlotKey) -> Error {
	Error {
		msg: format!("{:?} is not a valid handgun slot", slot),
		lvl: Level::Error,
	}
}

fn insert_fn(entity: &mut EntityCommands, slot: SlotKey) -> Result<(), Error> {
	let SlotKey::Hand(side) = slot else {
		return Err(slot_error(slot));
	};

	match side {
		Side::Right => entity.insert(Marker::<(HandGun, Right)>::new()),
		Side::Left => entity.insert(Marker::<(HandGun, Left)>::new()),
	};

	Ok(())
}

fn remove_fn(entity: &mut EntityCommands, slot: SlotKey) -> Result<(), Error> {
	let SlotKey::Hand(side) = slot else {
		return Err(slot_error(slot));
	};

	match side {
		Side::Right => entity.remove::<Marker<(HandGun, Right)>>(),
		Side::Left => entity.remove::<Marker<(HandGun, Left)>>(),
	};

	Ok(())
}

fn soft_override(running: &Skill<Active>, new: &Skill<Queued>) -> bool {
	running.marker == HANDGUN_META && new.marker == HANDGUN_META
}

const HANDGUN_META: MarkerMeta = MarkerMeta {
	insert_fn,
	remove_fn,
	soft_override,
};

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Side, SlotKey},
		markers::{Left, Right},
		systems::log::tests::{fake_log_error_lazy, FakeErrorLog},
		traits::marker::test_tools::{insert_lazy, remove_lazy},
	};
	use bevy::{
		app::{App, Update},
		ecs::system::IntoSystem,
		utils::default,
	};

	#[test]
	fn add_markers_right() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let marker = HandGun::marker();
		let slot = SlotKey::Hand(Side::Right);

		app.add_systems(
			Update,
			insert_lazy(marker, agent, slot).pipe(fake_log_error_lazy(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(true, false),
			(
				agent.contains::<Marker<(HandGun, Right)>>(),
				agent.contains::<FakeErrorLog>()
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
			insert_lazy(marker, agent, slot).pipe(fake_log_error_lazy(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(true, false),
			(
				agent.contains::<Marker<(HandGun, Left)>>(),
				agent.contains::<FakeErrorLog>()
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
			insert_lazy(marker, agent, slot).pipe(fake_log_error_lazy(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false, Some(slot_error(slot))),
			(
				agent.contains::<Marker<(HandGun, Left)>>(),
				agent.contains::<Marker<(HandGun, Right)>>(),
				agent.get::<FakeErrorLog>().map(|log| log.0.clone()),
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
			remove_lazy(marker, agent, slot).pipe(fake_log_error_lazy(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(
				agent.contains::<Marker<(HandGun, Right)>>(),
				agent.contains::<FakeErrorLog>()
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
			remove_lazy(marker, agent, slot).pipe(fake_log_error_lazy(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(
				agent.contains::<Marker<(HandGun, Left)>>(),
				agent.contains::<FakeErrorLog>()
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
			remove_lazy(marker, agent, slot).pipe(fake_log_error_lazy(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(slot_error(slot)),
			agent.get::<FakeErrorLog>().map(|l| l.0.clone())
		);
	}

	#[test]
	fn soft_override_true_when_both_skills_with_handgun_marker() {
		let handgun_marker = HandGun::marker();
		let running: Skill<Active> = Skill {
			marker: HandGun::marker(),
			..default()
		};
		let new: Skill<Queued> = Skill {
			marker: HandGun::marker(),
			..default()
		};

		assert!((handgun_marker.soft_override)(&running, &new));
	}

	#[test]
	fn soft_override_false_when_running_skill_not_with_handgun_marker() {
		let handgun_marker = HandGun::marker();
		let running: Skill<Active> = Skill {
			marker: default(),
			..default()
		};
		let new: Skill<Queued> = Skill {
			marker: HandGun::marker(),
			..default()
		};

		assert!(!(handgun_marker.soft_override)(&running, &new));
	}

	#[test]
	fn soft_override_false_when_new_skill_not_with_handgun_marker() {
		let handgun_marker = HandGun::marker();
		let running: Skill<Active> = Skill {
			marker: HandGun::marker(),
			..default()
		};
		let new: Skill<Queued> = Skill {
			marker: default(),
			..default()
		};

		assert!(!(handgun_marker.soft_override)(&running, &new));
	}
}
