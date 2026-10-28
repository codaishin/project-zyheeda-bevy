use crate::{
	components::action_key_interaction::ActionKeyInteraction,
	resources::mouse_override::MouseOverride,
};
use bevy::{prelude::*, ui_widgets::Activate};

impl MouseOverride {
	#[allow(clippy::type_complexity)]
	pub(crate) fn set_active(
		activate: On<Activate>,
		buttons: Query<(Entity, &ActionKeyInteraction)>,
		mut mouse_override: ResMut<Self>,
	) {
		if matches!(*mouse_override, Self::Active { input_state, .. } if input_state.is_some()) {
			return;
		}

		let Ok((panel, button)) = buttons.get(activate.entity) else {
			return;
		};

		*mouse_override = Self::Active {
			panel,
			action: button.action_key,
			input_state: None,
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resources::mouse_override::MouseOverride;
	use common::prelude::*;
	use testing::{SingleThreadedApp, fake_entity};

	#[derive(Resource, Debug, PartialEq)]
	struct _OverrideChanged(bool);

	fn setup(mouse_override: MouseOverride) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(mouse_override);
		app.add_observer(MouseOverride::set_active);
		app.add_systems(Update, |o: Res<MouseOverride>, mut commands: Commands| {
			commands.insert_resource(_OverrideChanged(o.is_changed()));
		});

		app
	}

	#[test]
	fn set_on_activate() {
		let mut app = setup(MouseOverride::Idle);
		let entity = app
			.world_mut()
			.spawn(ActionKeyInteraction {
				action_key: ActionKey::from(HandSlot::Left),
				override_active: false,
			})
			.trigger(|entity| Activate { entity })
			.id();

		app.update();

		assert_eq!(
			&MouseOverride::Active {
				panel: entity,
				action: ActionKey::from(HandSlot::Left),
				input_state: None
			},
			app.world().resource::<MouseOverride>()
		);
	}

	#[test]
	fn not_set_when_not_activated() {
		let mut app = setup(MouseOverride::Idle);
		app.world_mut().spawn(ActionKeyInteraction {
			action_key: ActionKey::default(),
			override_active: false,
		});

		app.update();

		assert_eq!(
			&MouseOverride::Idle,
			app.world().resource::<MouseOverride>()
		);
	}

	#[test]
	fn set_when_mouse_override_set_to_other_panel() {
		let mut app = setup(MouseOverride::Active {
			panel: fake_entity!(42),
			action: ActionKey::from(HandSlot::Right),
			input_state: None,
		});
		let entity = app
			.world_mut()
			.spawn(ActionKeyInteraction {
				action_key: ActionKey::from(HandSlot::Left),
				override_active: false,
			})
			.trigger(|entity| Activate { entity })
			.id();

		app.update();

		assert_eq!(
			&MouseOverride::Active {
				panel: entity,
				action: ActionKey::from(HandSlot::Left),
				input_state: None
			},
			app.world().resource::<MouseOverride>()
		);
	}

	#[test]
	fn do_nothing_when_mouse_override_set_to_active_and_has_input() {
		let mut app = setup(MouseOverride::Active {
			panel: fake_entity!(123),
			action: ActionKey::default(),
			input_state: Some(InputState::just_pressed()),
		});
		app.world_mut()
			.spawn(ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: false,
			})
			.trigger(|entity| Activate { entity });

		app.update();

		assert_eq!(
			&MouseOverride::Active {
				panel: fake_entity!(123),
				action: ActionKey::default(),
				input_state: Some(InputState::just_pressed()),
			},
			app.world().resource::<MouseOverride>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(MouseOverride::Idle);
		app.world_mut()
			.spawn(ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: false,
			})
			.trigger(|entity| Activate { entity });

		app.update();
		app.update();

		assert_eq!(
			&_OverrideChanged(false),
			app.world().resource::<_OverrideChanged>()
		);
	}

	#[test]
	fn do_nothing_when_quickbar_interaction_missing() {
		let mut app = setup(MouseOverride::Idle);
		app.world_mut()
			.spawn(())
			.trigger(|entity| Activate { entity });

		app.update();

		assert_eq!(
			&MouseOverride::Idle,
			app.world().resource::<MouseOverride>()
		);
	}
}
