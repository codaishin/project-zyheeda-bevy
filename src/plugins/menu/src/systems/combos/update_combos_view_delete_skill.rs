use crate::components::DeleteSkill;
use bevy::{prelude::*, ui::Interaction};
use common::tools::action_key::slot::Combo;

pub(crate) fn update_combos_view_delete_skill<TSkill>(
	deletes: Query<(&DeleteSkill, &Interaction)>,
) -> Combo<Option<TSkill>> {
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
	use common::tools::action_key::slot::{Side, SlotKey};
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Skill;

	fn setup(system: fn(In<Combo<Option<_Skill>>>)) -> App {
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
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			},
			Interaction::Pressed,
		));

		app.update();

		fn assert_combo(In(combo): In<Combo<Option<_Skill>>>) {
			assert_eq!(
				vec![(
					vec![
						SlotKey::BottomHand(Side::Left),
						SlotKey::BottomHand(Side::Right),
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
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			},
			Interaction::Hovered,
		));
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			},
			Interaction::None,
		));

		app.update();

		fn assert_combo_empty(In(combo): In<Combo<Option<_Skill>>>) {
			assert_eq!(vec![] as Combo<Option<_Skill>>, combo);
		}
	}
}
