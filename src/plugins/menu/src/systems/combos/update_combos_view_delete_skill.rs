use crate::components::DeleteSkill;
use bevy::{prelude::*, ui::Interaction};
use common::{tools::action_key::slot::PlayerSlot, traits::handles_combo_menu::Combo};

pub(crate) fn update_combos_view_delete_skill<TSkill>(
	deletes: Query<(&DeleteSkill, &Interaction)>,
) -> Combo<PlayerSlot, Option<TSkill>> {
	deletes
		.iter()
		.filter(pressed)
		.map(|(delete, ..)| (delete.key_path.clone(), None))
		.collect()
}

fn pressed((.., interaction): &(&DeleteSkill, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::slot::Side;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Skill;

	fn setup(system: fn(In<Combo<PlayerSlot, Option<_Skill>>>)) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			update_combos_view_delete_skill::<_Skill>.pipe(system),
		);

		app
	}

	#[test]
	fn return_combo_with_value_none() {
		let mut app = setup(assert_combo);
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![
					PlayerSlot::Lower(Side::Left),
					PlayerSlot::Lower(Side::Right),
				],
			},
			Interaction::Pressed,
		));

		app.update();

		fn assert_combo(In(combo): In<Combo<PlayerSlot, Option<_Skill>>>) {
			assert_eq!(
				vec![(
					vec![
						PlayerSlot::Lower(Side::Left),
						PlayerSlot::Lower(Side::Right),
					],
					None,
				)],
				combo
			);
		}
	}

	#[test]
	fn do_nothing_if_not_pressed() {
		let mut app = setup(assert_combo_empty);
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![
					PlayerSlot::Lower(Side::Left),
					PlayerSlot::Lower(Side::Right),
				],
			},
			Interaction::Hovered,
		));
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![
					PlayerSlot::Lower(Side::Left),
					PlayerSlot::Lower(Side::Right),
				],
			},
			Interaction::None,
		));

		app.update();

		fn assert_combo_empty(In(combo): In<Combo<PlayerSlot, Option<_Skill>>>) {
			assert_eq!(vec![] as Combo<PlayerSlot, Option<_Skill>>, combo);
		}
	}
}
