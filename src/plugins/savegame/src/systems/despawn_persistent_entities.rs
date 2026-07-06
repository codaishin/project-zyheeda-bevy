use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl<T> DespawnAll for T where T: Component {}

pub(crate) trait DespawnAll: Component + Sized {
	fn despawn_all(mut commands: ZyheedaCommands, targets: Query<Entity, With<Self>>) {
		for target in targets {
			commands.try_apply_on(&target, |e| e.try_despawn());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Component;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _Component::despawn_all);

		app
	}

	#[test]
	fn despawn_all_entities() {
		let mut app = setup();
		let entities = [
			app.world_mut().spawn(_Component).id(),
			app.world_mut().spawn(_Component).id(),
			app.world_mut().spawn(_Component).id(),
			app.world_mut().spawn(_Component).id(),
		];

		app.update();

		assert_eq!(
			[true, true, true, true],
			entities.map(|e| app.world().get_entity(e).is_err())
		);
	}

	#[test]
	fn do_not_despawn_entities_without_component() {
		let mut app = setup();
		let entities = [
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
		];

		app.update();

		assert_eq!(
			[false, false, false, false],
			entities.map(|e| app.world().get_entity(e).is_err())
		);
	}
}
