use crate::components::DeleteSkill;
use bevy::{ecs::component::Mutable, prelude::*, ui::Interaction};
use common::{
	tools::action_key::slot::SlotKey,
	traits::handles_loadout::combos_component::UpdateCombos,
};

impl DeleteSkill {
	pub(crate) fn from_combos<TAgent, TCombos>(
		deletes: Query<(&DeleteSkill, &Interaction)>,
		mut combos: Query<&mut TCombos, With<TAgent>>,
	) where
		TAgent: Component,
		TCombos: Component<Mutability = Mutable> + UpdateCombos<TKey = SlotKey>,
	{
		for mut combos in &mut combos {
			let deletes = deletes
				.iter()
				.filter(pressed)
				.map(|(delete, ..)| (delete.key_path.clone(), None))
				.collect::<Vec<_>>();
			if deletes.is_empty() {
				continue;
			}
			combos.update_combos(deletes);
		}
	}
}

fn pressed((.., interaction): &(&DeleteSkill, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::action_key::slot::PlayerSlot,
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

	impl LoadoutItem for _Combos {
		type TItem = _Skill;
	}

	impl LoadoutKey for Mock_Combos {
		type TKey = SlotKey;
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

	#[derive(Debug, PartialEq)]
	pub struct _Skill;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, DeleteSkill::from_combos::<_Agent, _Combos>);

		app
	}

	#[test]
	fn set_combo_with_value_none() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_update_combos()
					.times(1)
					.with(eq(vec![(
						vec![
							SlotKey::from(PlayerSlot::LOWER_L),
							SlotKey::from(PlayerSlot::LOWER_R),
						],
						None,
					)]))
					.return_const(());
			}),
		));
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![
					SlotKey::from(PlayerSlot::LOWER_L),
					SlotKey::from(PlayerSlot::LOWER_R),
				],
			},
			Interaction::Pressed,
		));

		app.update();
	}

	#[test]
	fn do_nothing_if_not_pressed() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_update_combos().never();
			}),
		));
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![
					SlotKey::from(PlayerSlot::LOWER_L),
					SlotKey::from(PlayerSlot::LOWER_R),
				],
			},
			Interaction::None,
		));

		app.update();
	}

	#[test]
	fn do_nothing_if_no_agent() {
		let mut app = setup();
		app.world_mut().spawn(_Combos::new().with_mock(|mock| {
			mock.expect_update_combos().never();
		}));
		app.world_mut().spawn((
			DeleteSkill {
				key_path: vec![
					SlotKey::from(PlayerSlot::LOWER_L),
					SlotKey::from(PlayerSlot::LOWER_R),
				],
			},
			Interaction::Pressed,
		));

		app.update();
	}
}
