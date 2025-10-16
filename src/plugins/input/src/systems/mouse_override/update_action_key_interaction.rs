use crate::{
	components::action_key_interaction::ActionKeyInteraction,
	resources::mouse_override::MouseOverride,
};
use bevy::prelude::*;

impl MouseOverride {
	pub(crate) fn update_action_key_interaction(
		mut interactions: Query<(Entity, &mut ActionKeyInteraction)>,
		mouse_override: Res<Self>,
	) {
		if !mouse_override.is_changed() {
			return;
		}

		match *mouse_override {
			MouseOverride::Idle => {
				for (_, mut interaction) in &mut interactions {
					interaction.override_active = false;
				}
			}
			MouseOverride::Active { panel, .. } => {
				for (entity, mut interaction) in &mut interactions {
					interaction.override_active = entity == panel;
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::ActionKey;
	use std::ops::DerefMut;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<MouseOverride>();
		app.add_systems(
			Update,
			(
				MouseOverride::update_action_key_interaction,
				IsChanged::<ActionKeyInteraction>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn set_active_from_mouse_override() {
		let mut app = setup();
		let panel = app
			.world_mut()
			.spawn(ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: false,
			})
			.id();
		app.insert_resource(MouseOverride::Active {
			panel,
			action: ActionKey::default(),
			input_state: None,
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
	fn set_others_to_not_active() {
		let mut app = setup();
		let entities = [
			app.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: true,
				})
				.id(),
			app.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: true,
				})
				.id(),
			app.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: true,
				})
				.id(),
		];
		app.insert_resource(MouseOverride::Active {
			panel: entities[1],
			action: ActionKey::default(),
			input_state: None,
		});

		app.update();

		assert_eq!(
			vec![
				&ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: false,
				},
				&ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: true,
				},
				&ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: false,
				},
			],
			app.world()
				.entity(entities)
				.iter()
				.filter_map(|e| e.get::<ActionKeyInteraction>())
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn set_all_to_false_if_idle() {
		let mut app = setup();
		let entities = [
			app.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: true,
				})
				.id(),
			app.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: true,
				})
				.id(),
			app.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: true,
				})
				.id(),
		];
		app.insert_resource(MouseOverride::Idle);

		app.update();

		assert_eq!(
			vec![
				&ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: false,
				},
				&ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: false,
				},
				&ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: false,
				},
			],
			app.world()
				.entity(entities)
				.iter()
				.filter_map(|e| e.get::<ActionKeyInteraction>())
				.collect::<Vec<_>>()
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
		app.insert_resource(MouseOverride::Active {
			panel,
			action: ActionKey::default(),
			input_state: None,
		});

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
		app.insert_resource(MouseOverride::Active {
			panel,
			action: ActionKey::default(),
			input_state: None,
		});

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
