use crate::components::button_state::UiButtonState;
use bevy::prelude::*;
use common::prelude::*;

impl UiButtonState {
	pub(crate) fn track(buttons: Query<(&Interaction, &mut Self)>) {
		for (interaction, mut state) in buttons {
			let new_state = match (state.0, interaction) {
				(InputState::Released { .. }, Interaction::Pressed) => InputState::just_pressed(),
				(InputState::Released { .. }, ..) => InputState::released(),
				(InputState::Pressed { .. }, Interaction::Pressed) => InputState::pressed(),
				(InputState::Pressed { .. }, ..) => InputState::just_released(),
			};

			if new_state == state.0 {
				continue;
			}

			state.0 = new_state;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(UiButtonState::track, IsChanged::<UiButtonState>::detect),
		);

		app
	}

	#[test]
	fn set_just_pressed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Interaction::Pressed,
				UiButtonState(InputState::Released { just_now: false }),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&UiButtonState(InputState::Pressed { just_now: true })),
			app.world().entity(entity).get::<UiButtonState>(),
		);
	}

	#[test]
	fn set_pressed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Interaction::Pressed,
				UiButtonState(InputState::Pressed { just_now: true }),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&UiButtonState(InputState::Pressed { just_now: false })),
			app.world().entity(entity).get::<UiButtonState>(),
		);
	}

	#[test]
	fn stay_pressed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Interaction::Pressed,
				UiButtonState(InputState::Pressed { just_now: false }),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&UiButtonState(InputState::Pressed { just_now: false })),
			app.world().entity(entity).get::<UiButtonState>(),
		);
	}

	#[test]
	fn no_change_when_staying_pressed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Interaction::Pressed,
				UiButtonState(InputState::Pressed { just_now: false }),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<UiButtonState>>(),
		);
	}

	#[test_case(Interaction::None; "no interaction")]
	#[test_case(Interaction::Hovered; "hovering")]
	fn set_just_released_when(interaction: Interaction) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				interaction,
				UiButtonState(InputState::Pressed { just_now: false }),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&UiButtonState(InputState::Released { just_now: true })),
			app.world().entity(entity).get::<UiButtonState>(),
		);
	}

	#[test_case(Interaction::None; "no interaction")]
	#[test_case(Interaction::Hovered; "hovering")]
	fn set_released_when(interaction: Interaction) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				interaction,
				UiButtonState(InputState::Released { just_now: true }),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&UiButtonState(InputState::Released { just_now: false })),
			app.world().entity(entity).get::<UiButtonState>(),
		);
	}

	#[test_case(Interaction::None; "no interaction")]
	#[test_case(Interaction::Hovered; "hovering")]
	fn stay_released_when(interaction: Interaction) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				interaction,
				UiButtonState(InputState::Released { just_now: false }),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&UiButtonState(InputState::Released { just_now: false })),
			app.world().entity(entity).get::<UiButtonState>(),
		);
	}
	#[test_case(Interaction::None; "no interaction")]
	#[test_case(Interaction::Hovered; "hovering")]
	fn no_change_when_staying_released_when(interaction: Interaction) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				interaction,
				UiButtonState(InputState::Released { just_now: false }),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<UiButtonState>>(),
		);
	}
}
