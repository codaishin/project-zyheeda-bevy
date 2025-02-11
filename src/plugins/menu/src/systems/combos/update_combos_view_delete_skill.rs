use crate::components::DeleteSkill;
use bevy::{prelude::*, ui::Interaction};
use common::{tools::slot_key::SlotKey, traits::handles_equipment::WriteItem};

pub(crate) fn update_combos_view_delete_skill<TAgent, TSkill, TCombos>(
	deletes: Query<(&DeleteSkill, &Interaction)>,
	mut agents: Query<&mut TCombos, With<TAgent>>,
) where
	TAgent: Component,
	TCombos: Component + WriteItem<Vec<SlotKey>, Option<TSkill>>,
{
	let Ok(mut combos) = agents.get_single_mut() else {
		return;
	};

	for (delete, _) in deletes.iter().filter(pressed) {
		combos.write_item(&delete.key_path, None);
	}
}

fn pressed((.., interaction): &(&DeleteSkill, &Interaction)) -> bool {
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

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq)]
	struct _Skill;

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
		fn write_item(&mut self, key: &Vec<SlotKey>, value: Option<_Skill>) {
			self.mock.write_item(key, value)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			update_combos_view_delete_skill::<_Agent, _Skill, _Combos>,
		);

		app
	}

	#[test]
	fn call_update_config_with_none() {
		let mut app = setup();
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			},
			Interaction::Pressed,
		));
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_write_item()
					.times(1)
					.with(
						eq(vec![
							SlotKey::BottomHand(Side::Left),
							SlotKey::BottomHand(Side::Right),
						]),
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
				key_path: vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
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
				key_path: vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			},
			Interaction::Pressed,
		));
		app.world_mut().spawn((_NoAgent, _Combos::default()));

		app.update();
	}
}
