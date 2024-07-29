use crate::components::DeleteSkill;
use bevy::{
	prelude::{Component, Query, Res, Resource, With},
	ui::Interaction,
};
use common::traits::map_value::TryMapBackwards;
use skills::{items::slot_key::SlotKey, skills::Skill, traits::UpdateConfig};

pub(crate) fn update_combos_view_delete_skill<
	TAgent: Component,
	TCombos: Component + UpdateConfig<Vec<SlotKey>, Option<Skill>>,
	TKey: Copy + Sync + Send + 'static,
	TMap: Resource + TryMapBackwards<TKey, SlotKey>,
>(
	map: Res<TMap>,
	deletes: Query<(&DeleteSkill<TKey>, &Interaction)>,
	mut agents: Query<&mut TCombos, With<TAgent>>,
) {
	let Ok(mut combos) = agents.get_single_mut() else {
		return;
	};

	let map = map.as_ref();

	for key_path in deletes.iter().filter(pressed).filter_map(slot_keys(map)) {
		combos.update_config(&key_path, None);
	}
}

fn pressed<TKey>((.., interaction): &(&DeleteSkill<TKey>, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

fn slot_keys<TKey: Copy, TMap: TryMapBackwards<TKey, SlotKey>>(
	map: &TMap,
) -> impl Fn((&DeleteSkill<TKey>, &Interaction)) -> Option<Vec<SlotKey>> + '_ {
	|(delete_skill, ..)| {
		let key_path = delete_skill
			.key_path
			.iter()
			.filter_map(|key| map.try_map_backwards(*key))
			.collect::<Vec<_>>();

		match key_path.len() == delete_skill.key_path.len() {
			true => Some(key_path),
			false => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::Component,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use mockall::{automock, predicate::eq};
	use skills::{skills::Skill, traits::UpdateConfig};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Default)]
	struct _Combos {
		mock: Mock_Combos,
	}

	#[automock]
	impl UpdateConfig<Vec<SlotKey>, Option<Skill>> for _Combos {
		fn update_config(&mut self, key: &Vec<SlotKey>, value: Option<Skill>) {
			self.mock.update_config(key, value)
		}
	}

	#[derive(Clone, Copy)]
	enum _Key {
		Left,
		Right,
		Unmapped,
	}

	#[derive(Resource, Default)]
	struct _Map;

	impl TryMapBackwards<_Key, SlotKey> for _Map {
		fn try_map_backwards(&self, value: _Key) -> Option<SlotKey> {
			match value {
				_Key::Left => Some(SlotKey::Hand(Side::Off)),
				_Key::Right => Some(SlotKey::Hand(Side::Main)),
				_Key::Unmapped => None,
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Map>();
		app.add_systems(
			Update,
			update_combos_view_delete_skill::<_Agent, _Combos, _Key, _Map>,
		);

		app
	}

	#[test]
	fn call_update_config_with_none() {
		let mut app = setup();
		let mut combos = _Combos::default();
		combos
			.mock
			.expect_update_config()
			.times(1)
			.with(
				eq(vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)]),
				eq(None),
			)
			.return_const(());

		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![_Key::Left, _Key::Right],
			},
			Interaction::Pressed,
		));
		app.world_mut().spawn((_Agent, combos));

		app.update();
	}

	#[test]
	fn do_nothing_if_not_all_keys_mappable() {
		let mut app = setup();
		let mut combos = _Combos::default();
		combos.mock.expect_update_config().never().return_const(());

		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![_Key::Left, _Key::Right, _Key::Unmapped],
			},
			Interaction::Pressed,
		));
		app.world_mut().spawn((_Agent, combos));

		app.update();
	}

	#[test]
	fn do_nothing_if_not_pressed() {
		let mut app = setup();
		let mut combos = _Combos::default();
		combos.mock.expect_update_config().never().return_const(());

		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![_Key::Left, _Key::Right],
			},
			Interaction::Hovered,
		));
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![_Key::Left, _Key::Right],
			},
			Interaction::None,
		));
		app.world_mut().spawn((_Agent, combos));

		app.update();
	}

	#[test]
	fn do_nothing_if_agent_missing() {
		#[derive(Component)]
		struct _NoAgent;

		let mut app = setup();
		let mut combos = _Combos::default();
		combos.mock.expect_update_config().never().return_const(());

		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![_Key::Left, _Key::Right],
			},
			Interaction::Pressed,
		));
		app.world_mut().spawn((_NoAgent, combos));

		app.update();
	}
}
