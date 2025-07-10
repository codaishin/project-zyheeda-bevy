use crate::components::ui_disabled::UIDisabled;
use bevy::prelude::*;
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};

impl UIDisabled {
	pub(crate) fn apply(
		mut commands: Commands,
		mut removed_disables: RemovedComponents<UIDisabled>,
		disabled: Query<Entity, (With<UIDisabled>, With<Interaction>)>,
		can_be_interactive: Query<(), With<CanBeInteractive>>,
	) {
		for entity in &disabled {
			commands.try_remove_from::<Interaction>(entity);
			commands.try_insert_on(entity, CanBeInteractive);
		}

		for entity in removed_disables.read() {
			if !can_be_interactive.contains(entity) {
				continue;
			}

			commands.try_insert_on(entity, Interaction::None);
		}
	}
}

#[derive(Component)]
pub(crate) struct CanBeInteractive;

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, UIDisabled::apply);

		app
	}

	#[test]
	fn remove_interaction() {
		let mut app = setup();
		let entity = app.world_mut().spawn((Interaction::None, UIDisabled)).id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<Interaction>());
	}

	#[test]
	fn do_not_remove_interaction_if_not_disabled() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Interaction::None).id();

		app.update();

		assert_eq!(
			Some(&Interaction::None),
			app.world().entity(entity).get::<Interaction>(),
		);
	}

	#[test]
	fn reinsert_interaction() {
		let mut app = setup();
		let entity = app.world_mut().spawn((Interaction::None, UIDisabled)).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<UIDisabled>();
		app.update();

		assert_eq!(
			Some(&Interaction::None),
			app.world().entity(entity).get::<Interaction>(),
		);
	}

	#[test]
	fn do_not_reinsert_interaction_if_entity_never_had_interaction() {
		let mut app = setup();
		let entity = app.world_mut().spawn(UIDisabled).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<UIDisabled>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<Interaction>());
	}
}
