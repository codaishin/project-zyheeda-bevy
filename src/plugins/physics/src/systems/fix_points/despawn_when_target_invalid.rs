use crate::components::fix_points::Anchor;
use bevy::prelude::*;
use common::{
	traits::accessors::get::{Get, TryApplyOn},
	zyheeda_commands::ZyheedaCommands,
};

impl<TFilter> Anchor<TFilter>
where
	TFilter: 'static,
{
	pub(crate) fn despawn_when_target_invalid(
		mut commands: ZyheedaCommands,
		anchors: Query<(Entity, &Self)>,
	) {
		for (entity, anchor) in &anchors {
			if commands.get(&anchor.target).is_some() {
				continue;
			}

			commands.try_apply_on(&entity, |e| {
				e.try_despawn();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::fix_points::Always;
	use bevy::app::{App, Update};
	use common::{
		CommonPlugin,
		components::persistent_entity::PersistentEntity,
		traits::handles_skill_physics::SkillSpawner,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin);
		app.add_systems(Update, Anchor::<Always>::despawn_when_target_invalid);

		app
	}

	#[test]
	fn despawn_when_target_invalid() {
		let mut app = setup();
		let target = PersistentEntity::default();
		let anchor = app
			.world_mut()
			.spawn(Anchor::<Always>::to_target(target).on_spawner(SkillSpawner::Neutral))
			.id();

		app.update();

		assert!(app.world().get_entity(anchor).is_err());
	}

	#[test]
	fn no_despawn_when_target_valid() {
		let mut app = setup();
		let target = PersistentEntity::default();
		let anchor = app
			.world_mut()
			.spawn(Anchor::<Always>::to_target(target).on_spawner(SkillSpawner::Neutral))
			.id();
		app.world_mut().spawn(target);

		app.update();

		assert!(app.world().get_entity(anchor).is_ok());
	}
}
