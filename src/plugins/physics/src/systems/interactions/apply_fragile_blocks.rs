use crate::{
	components::{blockable::Blockable, blocker_types::BlockerTypes},
	resources::ongoing_interactions::OngoingInteractions,
};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, handles_physics::PhysicalObject::Fragile},
	zyheeda_commands::ZyheedaCommands,
};

pub(crate) fn apply_fragile_blocks(
	mut commands: ZyheedaCommands,
	ongoing_interactions: Res<OngoingInteractions>,
	fragiles: Query<(Entity, &Blockable)>,
	blockers: Query<&BlockerTypes>,
) {
	for (blocker, blocked) in &ongoing_interactions.targets {
		for blocked in blocked {
			let Some(fragile) = is_fragile(blocked, blocker, &fragiles, &blockers) else {
				continue;
			};

			commands.try_apply_on(&fragile, |e| e.try_despawn());
		}
	}
}

fn is_fragile(
	fragile: &Entity,
	blocker: &Entity,
	fragiles: &Query<(Entity, &Blockable)>,
	blockers: &Query<&BlockerTypes>,
) -> Option<Entity> {
	let BlockerTypes(blocker) = blockers.get(*blocker).ok()?;
	let Ok((entity, Blockable(Fragile { destroyed_by }))) = fragiles.get(*fragile) else {
		return None;
	};

	blocker.intersection(destroyed_by).next().map(|_| entity)
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::Units,
		traits::handles_physics::{PhysicalObject::Beam, physical_bodies::Blocker},
	};
	use std::collections::{HashMap, HashSet};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<OngoingInteractions>();
		app.add_systems(Update, apply_fragile_blocks);

		app
	}

	#[test]
	fn destroy_on_collision() {
		let mut app = setup();
		let fragile = app
			.world_mut()
			.spawn(Blockable(Fragile {
				destroyed_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Physical]))
			.id();
		app.insert_resource(OngoingInteractions {
			targets: HashMap::from([(blocker, HashSet::from([fragile]))]),
		});

		app.update();

		assert!(app.world().get_entity(fragile).is_err());
	}

	#[test]
	fn do_not_destroy_on_collision_if_not_fragile() {
		let mut app = setup();
		let fragile = app
			.world_mut()
			.spawn(Blockable(Beam {
				range: Units::from(1.),
				blocked_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Physical]))
			.id();
		app.insert_resource(OngoingInteractions {
			targets: HashMap::from([(blocker, HashSet::from([fragile]))]),
		});

		app.update();

		assert!(app.world().get_entity(fragile).is_ok());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_non_matching_blocker() {
		let mut app = setup();
		let fragile = app
			.world_mut()
			.spawn(Blockable(Fragile {
				destroyed_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Force]))
			.id();
		app.insert_resource(OngoingInteractions {
			targets: HashMap::from([(blocker, HashSet::from([fragile]))]),
		});

		app.update();

		assert!(app.world().get_entity(fragile).is_ok());
	}
}
