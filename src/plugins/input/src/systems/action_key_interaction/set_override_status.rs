use crate::{
	components::action_key_interaction::ActionKeyInteraction,
	resources::mouse_override::MouseOverride,
};
use bevy::prelude::*;

impl ActionKeyInteraction {
	pub(crate) fn set_override_status(
		mut interactions: Query<&mut Self>,
		mouse_override: Res<MouseOverride>,
	) {
		if !mouse_override.is_changed() {
			return;
		}

		let panel = match *mouse_override {
			MouseOverride::Idle => return,
			MouseOverride::Ui { panel } => panel,
			MouseOverride::World { panel, .. } => panel,
		};
		let Ok(mut panel) = interactions.get_mut(panel) else {
			return;
		};

		panel.override_active = true;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{tools::action_key::ActionKey, traits::handles_input::InputState};
	use std::ops::DerefMut;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<MouseOverride>();
		app.add_systems(
			Update,
			(
				ActionKeyInteraction::set_override_status,
				IsChanged::<ActionKeyInteraction>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn set_primed_from_mouse_override_ui() {
		let mut app = setup();
		let panel = app
			.world_mut()
			.spawn(ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: false,
			})
			.id();
		app.insert_resource(MouseOverride::Ui { panel });

		app.update();

		assert_eq!(
			Some(&ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: true,
			}),
			app.world().entity(panel).get::<ActionKeyInteraction>()
		);
	}

	#[test]
	fn set_primed_from_mouse_override_world() {
		let mut app = setup();
		let panel = app
			.world_mut()
			.spawn(ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: false,
			})
			.id();
		app.insert_resource(MouseOverride::World {
			panel,
			action: ActionKey::default(),
			input_state: InputState::pressed(),
		});

		app.update();

		assert_eq!(
			Some(&ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: true,
			}),
			app.world().entity(panel).get::<ActionKeyInteraction>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let panel = app
			.world_mut()
			.spawn(ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: false,
			})
			.id();
		app.insert_resource(MouseOverride::Ui { panel });

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(panel)
				.get::<IsChanged<ActionKeyInteraction>>()
		);
	}

	#[test]
	fn act_again_if_mouse_override_changed() {
		let mut app = setup();
		let panel = app
			.world_mut()
			.spawn(ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: false,
			})
			.id();
		app.insert_resource(MouseOverride::Ui { panel });

		app.update();
		app.world_mut().resource_mut::<MouseOverride>().deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world()
				.entity(panel)
				.get::<IsChanged<ActionKeyInteraction>>()
		);
	}
}
