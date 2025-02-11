use crate::components::skill_button::{DropdownItem, SkillButton};
use bevy::{prelude::*, ui::Interaction};
use common::{
	tools::slot_key::SlotKey,
	traits::{handles_equipment::WriteItem, thread_safe::ThreadSafe},
};

impl<T> DropdownSkillSelectClick for T {}

pub(crate) trait DropdownSkillSelectClick {
	fn dropdown_skill_select_click<TAgent, TSkill, TCombos>(
		mut agents: Query<&mut TCombos, With<TAgent>>,
		skill_selects: Query<(&SkillButton<DropdownItem<Self>, TSkill>, &Interaction)>,
	) where
		Self: ThreadSafe + Sized,
		TAgent: Component,
		TCombos: Component + WriteItem<Vec<SlotKey>, Option<TSkill>>,
		TSkill: ThreadSafe + Clone,
	{
		let Ok(mut combos) = agents.get_single_mut() else {
			return;
		};

		for (skill_descriptor, ..) in skill_selects.iter().filter(pressed) {
			combos.write_item(
				&skill_descriptor.key_path,
				Some(skill_descriptor.skill.clone()),
			);
		}
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
		tools::slot_key::Side,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Debug, PartialEq, Clone)]
	struct _Skill;

	struct _Layout;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	impl Default for _Combos {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_write_item().return_const(());
			})
		}
	}

	#[automock]
	impl WriteItem<Vec<SlotKey>, Option<_Skill>> for _Combos {
		fn write_item(&mut self, key: &Vec<SlotKey>, skill: Option<_Skill>) {
			self.mock.write_item(key, skill)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			_Layout::dropdown_skill_select_click::<_Agent, _Skill, _Combos>,
		);

		app
	}

	#[test]
	fn update_skill() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_write_item()
					.times(1)
					.with(eq(vec![SlotKey::BottomHand(Side::Left)]), eq(Some(_Skill)))
					.return_const(());
			}),
		));
		app.world_mut().spawn((
			SkillButton::<DropdownItem<_Layout>, _Skill>::new(
				_Skill,
				vec![SlotKey::BottomHand(Side::Left)],
			),
			Interaction::Pressed,
		));

		app.update();
	}

	#[test]
	fn do_not_update_skill_when_interaction_not_pressed() {
		let mut app = setup();
		app.world_mut().spawn((_Agent, _Combos::default()));
		app.world_mut().spawn((
			SkillButton::<DropdownItem<_Layout>, _Skill>::new(
				_Skill,
				vec![SlotKey::BottomHand(Side::Left)],
			),
			Interaction::Hovered,
		));
		app.world_mut().spawn((
			SkillButton::<DropdownItem<_Layout>, _Skill>::new(
				_Skill,
				vec![SlotKey::BottomHand(Side::Left)],
			),
			Interaction::None,
		));

		app.update();
	}

	#[test]
	fn do_not_update_skill_no_agent_present() {
		let mut app = setup();
		app.world_mut().spawn(_Combos::default());
		app.world_mut().spawn((
			SkillButton::<DropdownItem<_Layout>, _Skill>::new(
				_Skill,
				vec![SlotKey::BottomHand(Side::Left)],
			),
			Interaction::Pressed,
		));

		app.update();
	}
}
