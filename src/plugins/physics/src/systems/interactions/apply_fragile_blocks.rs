use crate::{
	components::{blockable::Blockable, blocker_types::BlockerTypes, collision_domains::Physical},
	resources::root_collisions::RootCollisions,
};
use bevy::prelude::*;
use common::prelude::*;

pub(crate) fn apply_fragile_blocks(
	mut commands: ZyheedaCommands,
	ongoing_interactions: Res<RootCollisions<Physical>>,
	fragiles: Query<(Entity, &Blockable)>,
	blockers: Query<(&BlockerTypes, Option<&PersistentEntity>)>,
) {
	for (blocker, blocked) in &ongoing_interactions {
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
	blockers: &Query<(&BlockerTypes, Option<&PersistentEntity>)>,
) -> Option<Entity> {
	let (BlockerTypes(blocker), persistent_blocker) = blockers.get(*blocker).ok()?;
	let (entity, blockable) = fragiles.get(*fragile).ok()?;

	if matches!(persistent_blocker, Some(blocker) if blockable.ignore == Some(*blocker)) {
		return None;
	}

	let PhysicalObject::Fragile { ref destroyed_by } = blockable.object else {
		return None;
	};

	blocker.intersection(destroyed_by).next().map(|_| entity)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashSet;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin::with_asset_loading(false));
		app.init_resource::<RootCollisions<Physical>>();
		app.add_systems(Update, apply_fragile_blocks);

		app
	}

	#[test]
	fn destroy_on_collision() {
		let mut app = setup();
		let fragile = app
			.world_mut()
			.spawn(Blockable::from(PhysicalObject::Fragile {
				destroyed_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Physical]))
			.id();
		app.insert_resource(RootCollisions::<Physical>::from_ongoing([(
			blocker,
			HashSet::from([fragile]),
		)]));

		app.update();

		assert!(app.world().get_entity(fragile).is_err());
	}

	#[test]
	fn do_not_destroy_on_collision_if_not_fragile() {
		let mut app = setup();
		let fragile = app
			.world_mut()
			.spawn(Blockable::from(PhysicalObject::Beam {
				range: Units::from(1.),
				blocked_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Physical]))
			.id();
		app.insert_resource(RootCollisions::<Physical>::from_ongoing([(
			blocker,
			HashSet::from([fragile]),
		)]));

		app.update();

		assert!(app.world().get_entity(fragile).is_ok());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_non_matching_blocker() {
		let mut app = setup();
		let fragile = app
			.world_mut()
			.spawn(Blockable::from(PhysicalObject::Fragile {
				destroyed_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Force]))
			.id();
		app.insert_resource(RootCollisions::<Physical>::from_ongoing([(
			blocker,
			HashSet::from([fragile]),
		)]));

		app.update();

		assert!(app.world().get_entity(fragile).is_ok());
	}

	#[test]
	fn do_not_destroy_on_collision_if_collided_with_ignored() {
		let mut app = setup();
		let ignored = PersistentEntity::default();
		let fragile = app
			.world_mut()
			.spawn(
				Blockable::from(PhysicalObject::Fragile {
					destroyed_by: [Blocker::Physical].into(),
				})
				.ignoring(ignored),
			)
			.id();
		let blocker = app
			.world_mut()
			.spawn((ignored, BlockerTypes::from([Blocker::Physical])))
			.id();
		app.insert_resource(RootCollisions::<Physical>::from_ongoing([(
			blocker,
			HashSet::from([fragile]),
		)]));

		app.update();

		assert!(app.world().get_entity(fragile).is_ok());
	}
}
