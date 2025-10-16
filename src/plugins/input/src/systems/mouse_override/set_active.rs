use crate::{
	components::action_key_interaction::ActionKeyInteraction,
	resources::mouse_override::MouseOverride,
};
use bevy::prelude::*;

impl MouseOverride {
	#[allow(clippy::type_complexity)]
	pub(crate) fn set_active(
		interactions: Query<(Entity, &ActionKeyInteraction, &Interaction), Changed<Interaction>>,
		mut mouse_override: ResMut<Self>,
	) {
		if matches!(*mouse_override, Self::Active { input_state, .. } if input_state.is_some()) {
			return;
		}

		for (panel, ActionKeyInteraction { action_key, .. }, interaction) in &interactions {
			if interaction != &Interaction::Pressed {
				continue;
			}

			*mouse_override = Self::Active {
				panel,
				action: *action_key,
				input_state: None,
			};
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resources::mouse_override::MouseOverride;
	use common::{
		tools::action_key::{ActionKey, slot::PlayerSlot},
		traits::handles_input::InputState,
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Resource, Debug, PartialEq)]
	struct _OverrideChanged(bool);

	fn setup(mouse_override: MouseOverride) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(mouse_override);
		app.add_systems(
			Update,
			(
				MouseOverride::set_active,
				|o: Res<MouseOverride>, mut commands: Commands| {
					commands.insert_resource(_OverrideChanged(o.is_changed()));
				},
			)
				.chain(),
		);

		app
	}

	#[test]
	fn set_on_press() {
		let mut app = setup(MouseOverride::Idle);
		let entity = app
			.world_mut()
			.spawn((
				Interaction::Pressed,
				ActionKeyInteraction {
					action_key: ActionKey::from(PlayerSlot::UPPER_L),
					override_active: false,
				},
			))
			.id();

		app.update();

		assert_eq!(
			&MouseOverride::Active {
				panel: entity,
				action: ActionKey::from(PlayerSlot::UPPER_L),
				input_state: None
			},
			app.world().resource::<MouseOverride>()
		);
	}

	#[test_case(Interaction::Hovered; "hovered")]
	#[test_case(Interaction::None; "none")]
	fn set_not_set_when_not_pressed(interaction: Interaction) {
		let mut app = setup(MouseOverride::Idle);
		app.world_mut().spawn((
			interaction,
			ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: false,
			},
		));

		app.update();

		assert_eq!(
			&MouseOverride::Idle,
			app.world().resource::<MouseOverride>()
		);
	}

	#[test]
	fn set_when_mouse_override_set_to_other_panel() {
		let mut app = setup(MouseOverride::Active {
			panel: Entity::from_raw(42),
			action: ActionKey::from(PlayerSlot::LOWER_R),
			input_state: None,
		});
		let entity = app
			.world_mut()
			.spawn((
				Interaction::Pressed,
				ActionKeyInteraction {
					action_key: ActionKey::from(PlayerSlot::UPPER_L),
					override_active: false,
				},
			))
			.id();

		app.update();

		assert_eq!(
			&MouseOverride::Active {
				panel: entity,
				action: ActionKey::from(PlayerSlot::UPPER_L),
				input_state: None
			},
			app.world().resource::<MouseOverride>()
		);
	}

	#[test]
	fn do_nothing_when_mouse_override_set_to_active_and_has_input() {
		let mut app = setup(MouseOverride::Active {
			panel: Entity::from_raw(123),
			action: ActionKey::default(),
			input_state: Some(InputState::just_pressed()),
		});
		app.world_mut().spawn((
			Interaction::Pressed,
			ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: false,
			},
		));

		app.update();

		assert_eq!(
			&MouseOverride::Active {
				panel: Entity::from_raw(123),
				action: ActionKey::default(),
				input_state: Some(InputState::just_pressed()),
			},
			app.world().resource::<MouseOverride>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(MouseOverride::Idle);
		app.world_mut().spawn((
			Interaction::Pressed,
			ActionKeyInteraction {
				action_key: ActionKey::default(),
				override_active: false,
			},
		));

		app.update();
		app.update();

		assert_eq!(
			&_OverrideChanged(false),
			app.world().resource::<_OverrideChanged>()
		);
	}

	#[test]
	fn act_again_if_interaction_changed() {
		let mut app = setup(MouseOverride::Idle);
		let entity = app
			.world_mut()
			.spawn((
				Interaction::Pressed,
				ActionKeyInteraction {
					action_key: ActionKey::default(),
					override_active: false,
				},
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Interaction>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			&_OverrideChanged(true),
			app.world().resource::<_OverrideChanged>()
		);
	}

	#[test]
	fn do_nothing_when_quickbar_interaction_missing() {
		let mut app = setup(MouseOverride::Idle);
		app.world_mut().spawn(Interaction::Pressed);

		app.update();

		assert_eq!(
			&MouseOverride::Idle,
			app.world().resource::<MouseOverride>()
		);
	}
}
