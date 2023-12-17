use super::GetMarkerMeta;
use crate::{
	components::{Active, Marker, Queued, Side, Skill, SlotKey},
	errors::{Error, Level},
	markers::{
		meta::{MarkerMeta, MarkerModifyFn, SkillModify},
		Dual,
		HandGun,
		Left,
		Right,
	},
};
use bevy::ecs::system::EntityCommands;

impl GetMarkerMeta for HandGun {
	fn marker() -> MarkerMeta {
		MarkerMeta {
			insert_fn: INSERT_SINGLE,
			remove_fn: REMOVE_SINGLE,
			skill_modify: SkillModify {
				modify_single_fn: modify_single,
				modify_dual_fn: modify_dual,
			},
		}
	}
}

const INSERT_SINGLE: MarkerModifyFn = insert_fn::<(HandGun, Left), (HandGun, Right)>;
const REMOVE_SINGLE: MarkerModifyFn = remove_fn::<(HandGun, Left), (HandGun, Right)>;
const INSERT_DUAL: MarkerModifyFn = insert_fn::<(HandGun, Left, Dual), (HandGun, Right, Dual)>;
const REMOVE_DUAL: MarkerModifyFn = remove_fn::<(HandGun, Left, Dual), (HandGun, Right, Dual)>;

fn slot_error(slot: SlotKey) -> Error {
	Error {
		msg: format!("{:?} is not a valid handgun slot", slot),
		lvl: Level::Error,
	}
}

fn insert_fn<TLeft: Send + Sync + 'static, TRight: Send + Sync + 'static>(
	entity: &mut EntityCommands,
	slot: SlotKey,
) -> Result<(), Error> {
	let SlotKey::Hand(side) = slot else {
		return Err(slot_error(slot));
	};

	match side {
		Side::Left => entity.insert(Marker::<TLeft>::new()),
		Side::Right => entity.insert(Marker::<TRight>::new()),
	};

	Ok(())
}

fn remove_fn<TLeft: Send + Sync + 'static, TRight: Send + Sync + 'static>(
	entity: &mut EntityCommands,
	slot: SlotKey,
) -> Result<(), Error> {
	let SlotKey::Hand(side) = slot else {
		return Err(slot_error(slot));
	};

	match side {
		Side::Left => entity.remove::<Marker<TLeft>>(),
		Side::Right => entity.remove::<Marker<TRight>>(),
	};

	Ok(())
}

fn modify_single(running: &mut Skill<Active>, new: &mut Skill<Queued>) {
	running.data.ignore_after_cast = false;
	new.marker.insert_fn = INSERT_SINGLE;
	new.marker.remove_fn = REMOVE_SINGLE;
}

fn modify_dual(running: &mut Skill<Active>, new: &mut Skill<Queued>) {
	running.data.ignore_after_cast = true;
	new.marker.insert_fn = INSERT_DUAL;
	new.marker.remove_fn = REMOVE_DUAL;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Side, SlotKey},
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
		let marker = MarkerMeta {
			insert_fn: insert_fn::<u32, f32>,
			..default()
		};
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
				agent.contains::<Marker<f32>>(),
				agent.contains::<FakeErrorLog>()
			)
		);
	}

	#[test]
	fn add_markers_left() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let marker = MarkerMeta {
			insert_fn: insert_fn::<u32, f32>,
			..default()
		};
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
				agent.contains::<Marker<u32>>(),
				agent.contains::<FakeErrorLog>()
			)
		);
	}

	#[test]
	fn add_markers_no_hand() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let marker = MarkerMeta {
			insert_fn: insert_fn::<u32, f32>,
			..default()
		};
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
				agent.contains::<Marker<u32>>(),
				agent.contains::<Marker<f32>>(),
				agent.get::<FakeErrorLog>().map(|log| log.0.clone()),
			)
		);
	}

	#[test]
	fn remove_markers_right() {
		let mut app = App::new();
		let agent = app.world.spawn(Marker::<f32>::new()).id();
		let marker = MarkerMeta {
			remove_fn: remove_fn::<u32, f32>,
			..default()
		};
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
				agent.contains::<Marker<f32>>(),
				agent.contains::<FakeErrorLog>()
			)
		);
	}

	#[test]
	fn remove_markers_left() {
		let mut app = App::new();
		let agent = app.world.spawn(Marker::<u32>::new()).id();
		let marker = MarkerMeta {
			remove_fn: remove_fn::<u32, f32>,
			..default()
		};
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
				agent.contains::<Marker<u32>>(),
				agent.contains::<FakeErrorLog>()
			)
		);
	}

	#[test]
	fn remove_markers_no_hand() {
		let mut app = App::new();
		let agent = app.world.spawn(()).id();
		let marker = MarkerMeta {
			remove_fn: remove_fn::<u32, f32>,
			..default()
		};
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
	fn modify_single() {
		let modify = HandGun::marker().skill_modify.modify_single_fn;
		let mut running = Skill {
			data: Active {
				ignore_after_cast: true,
				..default()
			},
			..default()
		};
		let mut new = Skill { ..default() };

		modify(&mut running, &mut new);

		assert_eq!(
			(INSERT_SINGLE as usize, REMOVE_SINGLE as usize, false,),
			(
				new.marker.insert_fn as usize,
				new.marker.remove_fn as usize,
				running.data.ignore_after_cast
			)
		);
	}

	#[test]
	fn modify_dual() {
		let modify = HandGun::marker().skill_modify.modify_dual_fn;
		let mut running = Skill { ..default() };
		let mut new = Skill { ..default() };

		modify(&mut running, &mut new);

		assert_eq!(
			(INSERT_DUAL as usize, REMOVE_DUAL as usize, true,),
			(
				new.marker.insert_fn as usize,
				new.marker.remove_fn as usize,
				running.data.ignore_after_cast
			)
		);
	}
}
