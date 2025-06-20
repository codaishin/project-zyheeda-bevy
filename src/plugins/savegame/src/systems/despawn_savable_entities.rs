use crate::components::save::Save;
use bevy::prelude::*;
use common::traits::try_despawn::TryDespawn;

impl Save {
	pub(crate) fn despawn_all(mut commands: Commands, targets: Query<Entity, With<Self>>) {
		for target in &targets {
			commands.try_despawn(target);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Save::despawn_all);

		app
	}

	#[test]
	fn despawn_all_save_entities() {
		let mut app = setup();
		let entities = [
			app.world_mut().spawn(Save).id(),
			app.world_mut().spawn(Save).id(),
			app.world_mut().spawn(Save).id(),
			app.world_mut().spawn(Save).id(),
		];

		app.update();

		assert_eq!(
			[true, true, true, true],
			entities.map(|e| app.world().get_entity(e).is_err())
		);
	}

	#[test]
	fn do_not_despawn_entities_without_save_component() {
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
