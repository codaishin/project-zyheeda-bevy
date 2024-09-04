use crate::components::skill_button::{DropdownItem, SkillButton};
use bevy::{
	prelude::{Component, Query, With},
	ui::Interaction,
};
use skills::{items::slot_key::SlotKey, skills::Skill, traits::UpdateConfig};

pub(crate) fn update_combo_skills<TAgent, TCombos, TLayout>(
	mut agents: Query<&mut TCombos, With<TAgent>>,
	skill_selects: Query<(&SkillButton<DropdownItem<TLayout>>, &Interaction)>,
) where
	TLayout: Sync + Send + 'static,
	TAgent: Component,
	TCombos: Component + UpdateConfig<Vec<SlotKey>, Option<Skill>>,
{
	let Ok(mut combos) = agents.get_single_mut() else {
		return;
	};

	for (skill_descriptor, ..) in skill_selects.iter().filter(pressed) {
		combos.update_config(
			&skill_descriptor.key_path,
			Some(skill_descriptor.skill.clone()),
		);
	}
}

fn pressed<T>((.., interaction): &(&T, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		utils::default,
	};
	use common::{
		components::Side,
		test_tools::utils::SingleThreadedApp,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use skills::skills::Skill;

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
				mock.expect_update_config().return_const(());
			})
		}
	}

	#[automock]
	impl UpdateConfig<Vec<SlotKey>, Option<Skill>> for _Combos {
		fn update_config(&mut self, key: &Vec<SlotKey>, skill: Option<Skill>) {
			self.mock.update_config(key, skill)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, update_combo_skills::<_Agent, _Combos, _Layout>);

		app
	}

	#[test]
	fn update_skill() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_update_config()
					.times(1)
					.with(
						eq(vec![SlotKey::Hand(Side::Off)]),
						eq(Some(Skill {
							name: "my skill".to_owned(),
							..default()
						})),
					)
					.return_const(());
			}),
		));
		app.world_mut().spawn((
			SkillButton::<DropdownItem<_Layout>>::new(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				vec![SlotKey::Hand(Side::Off)],
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
			SkillButton::<DropdownItem<_Layout>>::new(
				Skill::default(),
				vec![SlotKey::Hand(Side::Off)],
			),
			Interaction::Hovered,
		));
		app.world_mut().spawn((
			SkillButton::<DropdownItem<_Layout>>::new(
				Skill::default(),
				vec![SlotKey::Hand(Side::Off)],
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
			SkillButton::<DropdownItem<_Layout>>::new(
				Skill::default(),
				vec![SlotKey::Hand(Side::Off)],
			),
			Interaction::Pressed,
		));

		app.update();
	}
}
