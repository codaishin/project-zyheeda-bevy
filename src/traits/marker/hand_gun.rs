use super::GetMarkerMeta;
use crate::{
	components::{Active, Marker, Queued, Side, Skill, SlotKey},
	errors::{Error, Level},
	markers::{
		meta::{MarkerMeta, Tag},
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
			insert_fn: insert_fn::<(HandGun, Left), (HandGun, Right)>,
			remove_fn: remove_fn::<(HandGun, Left), (HandGun, Right)>,
			soft_override,
			tag: Some(Tag::HandGun),
		}
	}
}

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

fn soft_override(running: &mut Skill<Active>, new: &mut Skill<Queued>) -> bool {
	if running.marker.tag != Some(Tag::HandGun) || new.marker.tag != Some(Tag::HandGun) {
		running.data.ignore_after_cast = false;
		return false;
	}
	match (running.data.slot, new.data.slot) {
		(SlotKey::Hand(running_side), SlotKey::Hand(new_side)) if running_side != new_side => {
			new.marker.insert_fn = insert_fn::<(HandGun, Left, Dual), (HandGun, Right, Dual)>;
			new.marker.remove_fn = remove_fn::<(HandGun, Left, Dual), (HandGun, Right, Dual)>;
			running.data.ignore_after_cast = true;
		}
		_ => {
			running.data.ignore_after_cast = false;
		}
	}
	true
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
	fn soft_override_true_when_both_skills_with_handgun_marker_different_sides() {
		let handgun_marker = HandGun::marker();
		let mut running = Skill {
			marker: HandGun::marker(),
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				ignore_after_cast: false,
				..default()
			},
			..default()
		};
		let mut new = Skill {
			data: Queued {
				slot: SlotKey::Hand(Side::Right),
				..default()
			},
			marker: HandGun::marker(),
			..default()
		};

		assert_eq!(
			(
				true,
				true,
				MarkerMeta {
					insert_fn: insert_fn::<(HandGun, Left, Dual), (HandGun, Right, Dual)>,
					remove_fn: remove_fn::<(HandGun, Left, Dual), (HandGun, Right, Dual)>,
					soft_override,
					tag: Some(Tag::HandGun)
				}
			),
			(
				(handgun_marker.soft_override)(&mut running, &mut new),
				running.data.ignore_after_cast,
				new.marker,
			)
		);
	}

	#[test]
	fn soft_override_true_when_both_skills_with_handgun_marker_same_side() {
		let handgun_marker = HandGun::marker();
		let mut running = Skill {
			marker: HandGun::marker(),
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				ignore_after_cast: true,
				..default()
			},
			..default()
		};
		let mut new = Skill {
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: HandGun::marker(),
			..default()
		};

		assert_eq!(
			(true, false, HandGun::marker()),
			(
				(handgun_marker.soft_override)(&mut running, &mut new),
				running.data.ignore_after_cast,
				new.marker,
			)
		);
	}

	#[test]
	fn soft_override_true_when_both_skills_with_handgun_marker_single_not_hand_slot() {
		let handgun_marker = HandGun::marker();
		let mut running = Skill {
			marker: HandGun::marker(),
			data: Active {
				slot: SlotKey::Legs,
				ignore_after_cast: true,
				..default()
			},
			..default()
		};
		let mut new = Skill {
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: HandGun::marker(),
			..default()
		};

		assert_eq!(
			(true, false, HandGun::marker()),
			(
				(handgun_marker.soft_override)(&mut running, &mut new),
				running.data.ignore_after_cast,
				new.marker,
			)
		);
	}

	#[test]
	fn soft_override_false_when_running_skill_not_with_handgun_marker() {
		let mut handgun_marker = HandGun::marker();
		handgun_marker.tag = None;
		let mut running = Skill {
			marker: handgun_marker,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				ignore_after_cast: true,
				..default()
			},
			..default()
		};
		let mut new = Skill {
			data: Queued {
				slot: SlotKey::Hand(Side::Right),
				..default()
			},
			marker: HandGun::marker(),
			..default()
		};

		assert_eq!(
			(false, false, HandGun::marker()),
			(
				(handgun_marker.soft_override)(&mut running, &mut new),
				running.data.ignore_after_cast,
				new.marker,
			)
		);
	}

	#[test]
	fn soft_override_false_when_new_skill_not_with_handgun_marker() {
		let mut handgun_marker = HandGun::marker();
		handgun_marker.tag = None;
		let mut running = Skill {
			marker: HandGun::marker(),
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				ignore_after_cast: true,
				..default()
			},
			..default()
		};
		let mut new = Skill {
			data: Queued {
				slot: SlotKey::Hand(Side::Right),
				..default()
			},
			marker: handgun_marker,
			..default()
		};

		assert_eq!(
			(false, false, handgun_marker),
			(
				(handgun_marker.soft_override)(&mut running, &mut new),
				running.data.ignore_after_cast,
				new.marker,
			)
		);
	}
}
