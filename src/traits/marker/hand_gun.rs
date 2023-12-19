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
				update_single_fn: modify_single,
				update_dual_fn: modify_dual,
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

fn modify_single(running: &Skill<Active>, new: &Skill<Queued>) -> (Skill<Active>, Skill<Queued>) {
	let mut running = *running;
	let mut new = *new;

	running.data.ignore_after_cast = false;
	new.marker.insert_fn = INSERT_SINGLE;
	new.marker.remove_fn = REMOVE_SINGLE;

	(running, new)
}

fn modify_dual(running: &Skill<Active>, new: &Skill<Queued>) -> (Skill<Active>, Skill<Queued>) {
	let mut running = *running;
	let mut new = *new;

	if running.marker.insert_fn != INSERT_SINGLE && running.marker.insert_fn != INSERT_DUAL {
		return (running, new);
	}

	running.data.ignore_after_cast = true;
	new.marker.insert_fn = INSERT_DUAL;
	new.marker.remove_fn = REMOVE_DUAL;

	(running, new)
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
		let modify = HandGun::marker().skill_modify.update_single_fn;
		let running = Skill {
			data: Active {
				ignore_after_cast: true,
				..default()
			},
			..default()
		};
		let new = Skill { ..default() };

		let (running, new) = modify(&running, &new);

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
	fn modify_dual_dual_markers() {
		let modify = HandGun::marker().skill_modify.update_dual_fn;
		let running = Skill {
			data: Active {
				ignore_after_cast: false,
				..default()
			},
			marker: MarkerMeta {
				insert_fn: INSERT_SINGLE,
				remove_fn: REMOVE_SINGLE,
				..default()
			},
			..default()
		};
		let new = Skill {
			data: Queued { ..default() },
			marker: MarkerMeta {
				insert_fn: INSERT_SINGLE,
				remove_fn: REMOVE_SINGLE,
				..default()
			},
			..default()
		};

		assert_eq!(
			(
				Skill {
					data: Active {
						ignore_after_cast: true,
						..default()
					},
					marker: MarkerMeta {
						insert_fn: INSERT_SINGLE,
						remove_fn: REMOVE_SINGLE,
						..default()
					},
					..default()
				},
				Skill {
					data: Queued { ..default() },
					marker: MarkerMeta {
						insert_fn: INSERT_DUAL,
						remove_fn: REMOVE_DUAL,
						..default()
					},
					..default()
				}
			),
			modify(&running, &new)
		);
	}

	#[test]
	fn modify_dual_when_running_with_unknown_markers_then_no_modification() {
		let modify = HandGun::marker().skill_modify.update_dual_fn;
		let running = Skill::default();
		let new = Skill::default();

		assert_eq!((running, new), modify(&running, &new));
	}

	#[test]
	fn modify_dual_when_running_with_dual_markers_then_modification() {
		let modify = HandGun::marker().skill_modify.update_dual_fn;
		let running = Skill {
			data: Active {
				ignore_after_cast: false,
				..default()
			},
			marker: MarkerMeta {
				insert_fn: INSERT_SINGLE,
				remove_fn: REMOVE_SINGLE,
				..default()
			},
			..default()
		};
		let new = Skill {
			data: Queued { ..default() },
			marker: MarkerMeta {
				insert_fn: INSERT_DUAL,
				remove_fn: REMOVE_DUAL,
				..default()
			},
			..default()
		};

		assert_eq!(
			(
				Skill {
					data: Active {
						ignore_after_cast: true,
						..default()
					},
					marker: MarkerMeta {
						insert_fn: INSERT_SINGLE,
						remove_fn: REMOVE_SINGLE,
						..default()
					},
					..default()
				},
				Skill {
					data: Queued { ..default() },
					marker: MarkerMeta {
						insert_fn: INSERT_DUAL,
						remove_fn: REMOVE_DUAL,
						..default()
					},
					..default()
				}
			),
			modify(&running, &new)
		);
	}
}
