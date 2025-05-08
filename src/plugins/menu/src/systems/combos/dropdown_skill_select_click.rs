use crate::components::combo_skill_button::{ComboSkillButton, DropdownItem};
use bevy::{prelude::*, ui::Interaction};
use common::{tools::action_key::slot::Combo, traits::thread_safe::ThreadSafe};

impl<T> DropdownSkillSelectClick for T {}

pub(crate) trait DropdownSkillSelectClick {
	fn dropdown_skill_select_click<TSkill>(
		skill_buttons: Query<(&ComboSkillButton<DropdownItem<Self>, TSkill>, &Interaction)>,
	) -> Combo<Option<TSkill>>
	where
		Self: ThreadSafe + Sized,
		TSkill: ThreadSafe + Clone,
	{
		skill_buttons
			.iter()
			.filter(pressed)
			.map(|(button, ..)| (button.key_path.clone(), Some(button.skill.clone())))
			.collect::<Vec<_>>()
	}
}

fn pressed<T>((.., interaction): &(&T, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::action_key::slot::{Side, SlotKey},
	};

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill;

	struct _Layout;

	fn setup(system: fn(In<Combo<Option<_Skill>>>)) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			_Layout::dropdown_skill_select_click::<_Skill>.pipe(system),
		);

		app
	}

	#[test]
	fn update_skill() {
		let mut app = setup(assert_combo);
		app.world_mut().spawn((
			ComboSkillButton::<DropdownItem<_Layout>, _Skill>::new(
				_Skill,
				vec![SlotKey::BottomHand(Side::Left)],
			),
			Interaction::Pressed,
		));

		app.update();

		fn assert_combo(In(combos): In<Combo<Option<_Skill>>>) {
			assert_eq!(
				vec![(vec![SlotKey::BottomHand(Side::Left)], Some(_Skill))],
				combos
			);
		}
	}

	#[test]
	fn do_not_update_skill_when_interaction_not_pressed() {
		let mut app = setup(assert_no_combo);
		app.world_mut().spawn((
			ComboSkillButton::<DropdownItem<_Layout>, _Skill>::new(
				_Skill,
				vec![SlotKey::BottomHand(Side::Left)],
			),
			Interaction::Hovered,
		));
		app.world_mut().spawn((
			ComboSkillButton::<DropdownItem<_Layout>, _Skill>::new(
				_Skill,
				vec![SlotKey::BottomHand(Side::Left)],
			),
			Interaction::None,
		));

		app.update();

		fn assert_no_combo(In(combos): In<Combo<Option<_Skill>>>) {
			assert_eq!(vec![] as Combo<Option<_Skill>>, combos);
		}
	}
}
