use crate::components::combo_skill_button::{ComboSkillButton, DropdownItem};
use bevy::{ecs::system::StaticSystemParam, prelude::*, ui::Interaction};
use common::traits::{
	accessors::get::GetContextMut,
	handles_loadout::combos::{Combos, UpdateCombos},
	thread_safe::ThreadSafe,
};
use std::fmt::Debug;

impl<TLayout, TId> ComboSkillButton<DropdownItem<TLayout>, TId>
where
	TLayout: 'static,
	TId: Debug + PartialEq + Clone + ThreadSafe,
{
	pub(crate) fn update<TAgent, TLoadout>(
		skill_buttons: Query<(&Self, &Interaction)>,
		agents: Query<Entity, With<TAgent>>,
		mut param: StaticSystemParam<TLoadout>,
	) where
		TAgent: Component,
		TLoadout: for<'c> GetContextMut<Combos, TContext<'c>: UpdateCombos<TId>>,
	{
		for entity in &agents {
			let new_combos = skill_buttons
				.iter()
				.filter(pressed)
				.map(|(button, ..)| (button.key_path.clone(), Some(button.skill.id.clone())))
				.collect::<Vec<_>>();
			if new_combos.is_empty() {
				continue;
			}
			let Some(mut ctx) = TLoadout::get_context_mut(&mut param, Combos { entity }) else {
				continue;
			};

			ctx.update_combos(new_combos);
		}
	}
}

fn pressed<T>((.., interaction): &(&T, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::combo_overview::ComboSkill;
	use common::{
		tools::action_key::slot::{PlayerSlot, SlotKey},
		traits::{handles_loadout::combos::Combo, handles_localization::Token},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::sync::LazyLock;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	#[derive(Debug, PartialEq, Clone)]
	struct _Id;

	#[automock]
	impl UpdateCombos<_Id> for _Combos {
		fn update_combos(&mut self, combos: Combo<SlotKey, Option<_Id>>) {
			self.mock.update_combos(combos);
		}
	}

	struct _Layout;

	static SKILL: LazyLock<ComboSkill<_Id>> = LazyLock::new(|| ComboSkill {
		id: _Id,
		token: Token::default(),
		icon: Handle::default(),
	});

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			ComboSkillButton::<DropdownItem<_Layout>, _Id>::update::<_Agent, Query<&mut _Combos>>,
		);

		app
	}

	#[test]
	fn update_skill() {
		let mut app = setup();
		app.world_mut().spawn((
			ComboSkillButton::<DropdownItem<_Layout>, _Id>::new(
				SKILL.clone(),
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
						Some(_Id),
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
			ComboSkillButton::<DropdownItem<_Layout>, _Id>::new(
				SKILL.clone(),
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
			ComboSkillButton::<DropdownItem<_Layout>, _Id>::new(
				SKILL.clone(),
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
