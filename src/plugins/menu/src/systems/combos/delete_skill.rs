use crate::components::DeleteSkill;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::EntityContextMut,
	handles_loadout::combos::{Combos, UpdateCombos},
};

impl DeleteSkill {
	pub(crate) fn from_combos<TAgent, TLoadout, TId>(
		deletes: Query<(&DeleteSkill, &Interaction)>,
		agents: Query<Entity, With<TAgent>>,
		mut param: StaticSystemParam<TLoadout>,
	) where
		TAgent: Component,
		TLoadout: for<'c> EntityContextMut<Combos, TContext<'c>: UpdateCombos<TId>>,
	{
		for agent in &agents {
			let Some(mut ctx) = TLoadout::get_entity_context_mut(&mut param, agent, Combos) else {
				continue;
			};
			let deletes = deletes
				.iter()
				.filter(pressed)
				.map(|(delete, ..)| (delete.key_path.clone(), None))
				.collect::<Vec<_>>();
			if deletes.is_empty() {
				continue;
			}
			ctx.update_combos(deletes);
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
		tools::action_key::slot::{PlayerSlot, SlotKey},
		traits::handles_loadout::combos::Combo,
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

	#[automock]
	impl UpdateCombos<_Id> for _Combos {
		fn update_combos(&mut self, combos: Combo<SlotKey, Option<_Id>>) {
			self.mock.update_combos(combos);
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct _Id;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			DeleteSkill::from_combos::<_Agent, Query<&mut _Combos>, _Id>,
		);

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
