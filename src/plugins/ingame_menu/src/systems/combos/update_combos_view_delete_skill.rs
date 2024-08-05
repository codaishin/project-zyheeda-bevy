use crate::components::DeleteSkill;
use bevy::{
	prelude::{Component, Query, With},
	ui::Interaction,
};
use skills::{items::slot_key::SlotKey, skills::Skill, traits::UpdateConfig};

pub(crate) fn update_combos_view_delete_skill<
	TAgent: Component,
	TCombos: Component + UpdateConfig<Vec<SlotKey>, Option<Skill>>,
>(
	deletes: Query<(&DeleteSkill, &Interaction)>,
	mut agents: Query<&mut TCombos, With<TAgent>>,
) {
	let Ok(mut combos) = agents.get_single_mut() else {
		return;
	};

	for (delete, _) in deletes.iter().filter(pressed) {
		combos.update_config(&delete.key_path, None);
	}
}

fn pressed((.., interaction): &(&DeleteSkill, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::Component,
	};
	use common::{
		components::Side,
		test_tools::utils::SingleThreadedApp,
		traits::nested_mock::NestedMock,
	};
	use macros::NestedMock;
	use mockall::{automock, predicate::eq};
	use skills::{skills::Skill, traits::UpdateConfig};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMock)]
	struct _Combos {
		mock: Mock_Combos,
	}

	impl Default for _Combos {
		fn default() -> Self {
			Self::new_mock(|mock| {
				mock.expect_update_config().return_const(());
			})
		}
	}

	#[automock]
	impl UpdateConfig<Vec<SlotKey>, Option<Skill>> for _Combos {
		fn update_config(&mut self, key: &Vec<SlotKey>, value: Option<Skill>) {
			self.mock.update_config(key, value)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, update_combos_view_delete_skill::<_Agent, _Combos>);

		app
	}

	#[test]
	fn call_update_config_with_none() {
		let mut app = setup();
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
			},
			Interaction::Pressed,
		));
		app.world_mut().spawn((
			_Agent,
			_Combos::new_mock(|mock| {
				mock.expect_update_config()
					.times(1)
					.with(
						eq(vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)]),
						eq(None),
					)
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn do_nothing_if_not_all_keys_mappable() {
		let mut app = setup();
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
			},
			Interaction::Pressed,
		));
		app.world_mut().spawn((_Agent, _Combos::default()));

		app.update();
	}

	#[test]
	fn do_nothing_if_not_pressed() {
		let mut app = setup();
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
			},
			Interaction::Hovered,
		));
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
			},
			Interaction::None,
		));
		app.world_mut().spawn((_Agent, _Combos::default()));

		app.update();
	}

	#[test]
	fn do_nothing_if_agent_missing() {
		#[derive(Component)]
		struct _NoAgent;

		let mut app = setup();
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
			},
			Interaction::Pressed,
		));
		app.world_mut().spawn((_NoAgent, _Combos::default()));

		app.update();
	}
}
