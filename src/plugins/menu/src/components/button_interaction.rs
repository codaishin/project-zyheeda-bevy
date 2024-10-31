use crate::traits::is_released::IsReleased;
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;

#[derive(Component, Debug, PartialEq)]
pub(crate) enum ButtonInteraction {
	None { hovered: bool },
	Pressed,
	Released { hovered: bool },
}

impl From<&Interaction> for ButtonInteraction {
	fn from(interaction: &Interaction) -> Self {
		match interaction {
			Interaction::None => Self::None { hovered: false },
			Interaction::Pressed => Self::Pressed,
			Interaction::Hovered => Self::None { hovered: true },
		}
	}
}

impl ButtonInteraction {
	pub(crate) fn system(
		mut commands: Commands,
		buttons: Query<(Entity, &Interaction, Option<&ButtonInteraction>)>,
	) {
		for (entity, interaction, last_interaction) in &buttons {
			let interaction = match (last_interaction, Self::from(interaction)) {
				(Some(Self::Pressed), Self::None { hovered }) => Self::Released { hovered },
				(_, interaction) => interaction,
			};

			commands.try_insert_on(entity, interaction);
		}
	}
}

impl IsReleased for ButtonInteraction {
	fn is_released(&self) -> bool {
		matches!(self, Self::Released { hovered: true })
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, ButtonInteraction::system);

		app
	}

	#[test]
	fn set_from_interaction() {
		let mut app = setup();
		let entities = [
			app.world_mut().spawn(Interaction::None).id(),
			app.world_mut().spawn(Interaction::Pressed).id(),
			app.world_mut().spawn(Interaction::Hovered).id(),
		];

		app.update();

		assert_eq!(
			[
				Some(&ButtonInteraction::None { hovered: false }),
				Some(&ButtonInteraction::Pressed),
				Some(&ButtonInteraction::None { hovered: true }),
			],
			entities.map(|entity| app.world().entity(entity).get::<ButtonInteraction>()),
		);
	}

	#[test]
	fn set_to_released_without_hovering_when_pressed_then_none() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Interaction::Pressed).id();

		app.update();
		app.world_mut().entity_mut(entity).insert(Interaction::None);
		app.update();

		assert_eq!(
			Some(&ButtonInteraction::Released { hovered: false }),
			app.world().entity(entity).get::<ButtonInteraction>(),
		);
	}

	#[test]
	fn set_to_released_with_hovering_when_pressed_then_none() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Interaction::Pressed).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Interaction::Hovered);
		app.update();

		assert_eq!(
			Some(&ButtonInteraction::Released { hovered: true }),
			app.world().entity(entity).get::<ButtonInteraction>(),
		);
	}

	#[test]
	fn is_released_on_release_with_hover() {
		let interaction = ButtonInteraction::Released { hovered: true };

		assert!(interaction.is_released());
	}

	#[test]
	fn is_not_released_on_release_without_hover() {
		let interaction = ButtonInteraction::Released { hovered: false };

		assert!(!interaction.is_released());
	}

	#[test]
	fn is_not_released_on_pressed() {
		let interaction = ButtonInteraction::Pressed;

		assert!(!interaction.is_released());
	}

	#[test]
	fn is_not_released_on_none_without_hover() {
		let interaction = ButtonInteraction::None { hovered: false };

		assert!(!interaction.is_released());
	}

	#[test]
	fn is_not_released_on_none_with_hover() {
		let interaction = ButtonInteraction::None { hovered: true };

		assert!(!interaction.is_released());
	}
}
