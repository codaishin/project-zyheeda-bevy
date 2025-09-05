use crate::components::combo_skill_button::{ComboSkillButton, DropdownItem};
use bevy::{ecs::component::Mutable, prelude::*, ui::Interaction};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{handles_loadout::combos_component::UpdateCombos, thread_safe::ThreadSafe},
};

impl<TLayout, TSkill> ComboSkillButton<DropdownItem<TLayout>, TSkill>
where
	TLayout: ThreadSafe,
	TSkill: ThreadSafe + Clone,
{
	pub(crate) fn update<TAgent, TCombos>(
		skill_buttons: Query<(&Self, &Interaction)>,
		mut combos: Query<&mut TCombos, With<TAgent>>,
	) where
		TAgent: Component,
		TCombos: Component<Mutability = Mutable> + UpdateCombos<TKey = SlotKey, TItem = TSkill>,
	{
		for mut combos in &mut combos {
			let new_combos = skill_buttons
				.iter()
				.filter(pressed)
				.map(|(button, ..)| (button.key_path.clone(), Some(button.skill.clone())))
				.collect::<Vec<_>>();
			if new_combos.is_empty() {
				continue;
			}
			combos.update_combos(new_combos);
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
		tools::action_key::slot::{PlayerSlot, SlotKey},
		traits::handles_loadout::{
			combos_component::Combo,
			loadout::{LoadoutItem, LoadoutKey},
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	impl LoadoutKey for _Combos {
		type TKey = SlotKey;
	}

	impl LoadoutKey for Mock_Combos {
		type TKey = SlotKey;
	}

	impl LoadoutItem for _Combos {
		type TItem = _Skill;
	}

	impl LoadoutItem for Mock_Combos {
		type TItem = _Skill;
	}

	#[automock]
	impl UpdateCombos for _Combos {
		fn update_combos(&mut self, combos: Combo<SlotKey, Option<_Skill>>) {
			self.mock.update_combos(combos);
		}
	}

	#[derive(Debug, PartialEq, Default, Clone)]
	pub struct _Skill;

	struct _Layout;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			ComboSkillButton::<DropdownItem<_Layout>, _Skill>::update::<_Agent, _Combos>,
		);

		app
	}

	#[test]
	fn update_skill() {
		let mut app = setup();
		app.world_mut().spawn((
			ComboSkillButton::<DropdownItem<_Layout>, _Skill>::new(
				_Skill,
				vec![SlotKey::from(PlayerSlot::LOWER_L)],
			),
			Interaction::Pressed,
		));
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_update_combos()
					.times(1)
					.with(eq(vec![(
						vec![SlotKey::from(PlayerSlot::LOWER_L)],
						Some(_Skill),
					)]))
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_update_skill_when_interaction_not_pressed() {
		let mut app = setup();
		app.world_mut().spawn((
			ComboSkillButton::<DropdownItem<_Layout>, _Skill>::new(
				_Skill,
				vec![SlotKey::from(PlayerSlot::LOWER_L)],
			),
			Interaction::Hovered,
		));
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_update_combos().never();
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_update_skill_when_agent_missing_interaction_not_pressed() {
		let mut app = setup();
		app.world_mut().spawn((
			ComboSkillButton::<DropdownItem<_Layout>, _Skill>::new(
				_Skill,
				vec![SlotKey::from(PlayerSlot::LOWER_L)],
			),
			Interaction::Pressed,
		));
		app.world_mut().spawn(_Combos::new().with_mock(|mock| {
			mock.expect_update_combos().never();
		}));

		app.update();
	}
}
