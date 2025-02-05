use crate::components::key_select::{KeySelect, ReKeySkill};
use bevy::prelude::{Component, In, Query, With};
use common::{tools::slot_key::SlotKey, traits::handles_equipment::WriteItem};

pub(crate) fn update_combo_keys<TAgent, TCombos>(
	key_select: In<Option<KeySelect<ReKeySkill<SlotKey>, SlotKey>>>,
	mut agents: Query<&mut TCombos, With<TAgent>>,
) where
	TAgent: Component,
	TCombos: Component + WriteItem<Vec<SlotKey>, SlotKey>,
{
	let Some(key_select) = key_select.0 else {
		return;
	};
	let Ok(mut agent) = agents.get_single_mut() else {
		return;
	};

	agent.write_item(&key_select.key_path, key_select.extra.to);
}

#[cfg(test)]
mod test {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::IntoSystem,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::slot_key::Side,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	impl Default for _Combos {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_write_item().never().return_const(());
			})
		}
	}

	#[automock]
	impl WriteItem<Vec<SlotKey>, SlotKey> for _Combos {
		fn write_item(&mut self, key: &Vec<SlotKey>, value: SlotKey) {
			self.mock.write_item(key, value)
		}
	}

	fn setup(key_select: Option<KeySelect<ReKeySkill<SlotKey>, SlotKey>>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(move || key_select.clone()).pipe(update_combo_keys::<_Agent, _Combos>),
		);

		app
	}

	#[test]
	fn call_update_config() {
		let mut app = setup(Some(KeySelect {
			extra: ReKeySkill {
				to: SlotKey::BottomHand(Side::Right),
			},
			key_path: vec![
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left),
			],
		}));

		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_write_item()
					.times(1)
					.with(
						eq(vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
						]),
						eq(SlotKey::BottomHand(Side::Right)),
					)
					.return_const(());
			}),
		));
		app.update();
	}

	#[test]
	fn don_call_update_config_when_no_agent() {
		#[derive(Component)]
		struct _NonAgent;

		let mut app = setup(Some(KeySelect {
			extra: ReKeySkill {
				to: SlotKey::BottomHand(Side::Right),
			},
			key_path: vec![
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left),
			],
		}));

		app.world_mut().spawn((_NonAgent, _Combos::default()));
		app.update();
	}
}
