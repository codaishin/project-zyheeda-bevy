use crate::components::ui_disabled::UIDisabled;
use bevy::{prelude::*, ui_widgets::Button};
use common::prelude::*;

impl UIDisabled {
	pub(crate) fn apply(
		mut commands: ZyheedaCommands,
		mut removed_disables: RemovedComponents<UIDisabled>,
		disabled: Query<Entity, (With<UIDisabled>, With<Button>)>,
		can_be_interactive: Query<(), With<CanBeButton>>,
	) {
		for entity in &disabled {
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<Button>();
				e.try_insert(CanBeButton);
			});
		}

		for entity in removed_disables.read() {
			if !can_be_interactive.contains(entity) {
				continue;
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(Button);
			});
		}
	}
}

#[derive(Component)]
pub(crate) struct CanBeButton;

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
	fn remove_button() {
		let mut app = setup();
		let entity = app.world_mut().spawn((Button, UIDisabled)).id();

		app.update();

		assert!(!app.world().entity(entity).contains::<Button>());
	}

	#[test]
	fn do_not_remove_button_if_not_disabled() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Button).id();

		app.update();

		assert!(app.world().entity(entity).contains::<Button>());
	}

	#[test]
	fn reinsert_button() {
		let mut app = setup();
		let entity = app.world_mut().spawn((Button, UIDisabled)).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<UIDisabled>();
		app.update();

		assert!(app.world().entity(entity).contains::<Button>());
	}

	#[test]
	fn do_not_reinsert_button_if_entity_never_had_interaction() {
		let mut app = setup();
		let entity = app.world_mut().spawn(UIDisabled).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<UIDisabled>();
		app.update();

		assert!(!app.world().entity(entity).contains::<Button>());
	}
}
