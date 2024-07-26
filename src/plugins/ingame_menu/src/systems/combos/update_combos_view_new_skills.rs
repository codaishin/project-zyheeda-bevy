use crate::components::{key_select::KeySelect, SkillSelectDropdownCommand};
use bevy::{
	prelude::{Commands, Mut, Parent, Query},
	text::Text,
	ui::Interaction,
};
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn update_combos_view_new_skills(
	mut commands: Commands,
	key_selects: Query<(&KeySelect, &Interaction)>,
	mut texts: Query<(&mut Text, &Parent)>,
) {
	for (key_select, ..) in key_selects.iter().filter(pressed) {
		insert_skill_dropdown(&mut commands, key_select);
		set_skill_label(&mut texts, key_select);
	}
}

fn pressed((.., interaction): &(&KeySelect, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

fn insert_skill_dropdown(commands: &mut Commands, key_select: &KeySelect) {
	commands.try_insert_on(
		key_select.skill_button,
		SkillSelectDropdownCommand {
			key_path: key_select.key_path.clone(),
		},
	);
}

fn set_skill_label(texts: &mut Query<(&mut Text, &Parent)>, key_select: &KeySelect) -> Option<()> {
	let (mut text, ..) = get_text(texts, key_select)?;
	let section = text.sections.get_mut(0)?;

	"+".clone_into(&mut section.value);

	Some(())
}

fn get_text<'a>(
	texts: &'a mut Query<(&mut Text, &Parent)>,
	key_select: &'a KeySelect,
) -> Option<(Mut<'a, Text>, &'a Parent)> {
	texts
		.iter_mut()
		.find(|(_, parent)| parent.get() == key_select.skill_button)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::SkillSelectDropdownCommand;
	use bevy::{
		app::{App, Update},
		prelude::{BuildWorldChildren, Entity, KeyCode, TextBundle},
		text::Text,
		utils::default,
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, update_combos_view_new_skills);

		app
	}

	#[test]
	fn insert_skill_select_dropdown_command() {
		let mut app = setup();
		let skill_button = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Interaction::Pressed,
			KeySelect {
				skill_button,
				key_button: Entity::from_raw(101),
				key_path: vec![KeyCode::KeyA, KeyCode::KeyB],
			},
		));

		app.update();

		let skill_button = app.world().entity(skill_button);

		assert_eq!(
			Some(&SkillSelectDropdownCommand {
				key_path: vec![KeyCode::KeyA, KeyCode::KeyB]
			}),
			skill_button.get::<SkillSelectDropdownCommand<KeyCode>>(),
		)
	}

	#[test]
	fn do_nothing_when_not_interaction_pressed() {
		let mut app = setup();
		let skill_button = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Interaction::Hovered,
			KeySelect {
				skill_button,
				key_button: Entity::from_raw(101),
				key_path: vec![KeyCode::KeyA, KeyCode::KeyB],
			},
		));
		app.world_mut().spawn((
			Interaction::None,
			KeySelect {
				skill_button,
				key_button: Entity::from_raw(101),
				key_path: vec![KeyCode::KeyA, KeyCode::KeyB],
			},
		));

		app.update();

		let skill_button = app.world().entity(skill_button);

		assert_eq!(
			None,
			skill_button.get::<SkillSelectDropdownCommand<KeyCode>>(),
		)
	}

	#[test]
	fn set_skill_button_text_to_plus() {
		let mut app = setup();
		let skill_button = app.world_mut().spawn_empty().id();
		let text = app
			.world_mut()
			.spawn(TextBundle::from_section("", default()))
			.set_parent(skill_button)
			.id();
		app.world_mut().spawn((
			Interaction::Pressed,
			KeySelect {
				skill_button,
				key_button: Entity::from_raw(101),
				key_path: vec![KeyCode::KeyA, KeyCode::KeyB],
			},
		));

		app.update();

		let text = app.world().entity(text).get::<Text>().unwrap();

		assert_eq!("+", text.sections[0].value)
	}
}
