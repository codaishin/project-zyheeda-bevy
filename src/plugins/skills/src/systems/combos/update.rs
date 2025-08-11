use crate::{skills::Skill, traits::write_item::WriteItem};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{tools::action_key::slot::SlotKey, traits::handles_combo_menu::Combo};

impl<T> UpdateCombos for T where
	T: Component<Mutability = Mutable> + WriteItem<Vec<SlotKey>, Option<Skill>>
{
}

pub(crate) trait UpdateCombos:
	Component<Mutability = Mutable> + WriteItem<Vec<SlotKey>, Option<Skill>> + Sized
{
	fn update_for<TAgent, TKey>(
		In(updated_combos): In<Combo<TKey, Option<Skill>>>,
		mut combos: Query<&mut Self, With<TAgent>>,
	) where
		TAgent: Component,
		TKey: Into<SlotKey>,
	{
		let Ok(mut combos) = combos.single_mut() else {
			return;
		};

		for (combo_keys, skill) in updated_combos {
			let keys = combo_keys.into_iter().map(|key| key.into()).collect();
			combos.write_item(&keys, skill);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::action_key::slot::{PlayerSlot, Side},
		traits::handles_localization::Token,
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
	impl WriteItem<Vec<SlotKey>, Option<Skill>> for _Combos {
		fn write_item(&mut self, key: &Vec<SlotKey>, value: Option<Skill>) {
			self.mock.write_item(key, value);
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn call_write_item() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_write_item()
					.times(1)
					.with(
						eq(vec![SlotKey::from(PlayerSlot::Upper(Side::Left))]),
						eq(Some(Skill {
							token: Token::from("my skill"),
							..default()
						})),
					)
					.return_const(());
				mock.expect_write_item()
					.times(1)
					.with(
						eq(vec![SlotKey::from(PlayerSlot::Upper(Side::Right))]),
						eq(None),
					)
					.return_const(());
			}),
		));

		app.world_mut().run_system_once_with(
			_Combos::update_for::<_Agent, PlayerSlot>,
			vec![
				(
					vec![PlayerSlot::Upper(Side::Left)],
					Some(Skill {
						token: Token::from("my skill"),
						..default()
					}),
				),
				(vec![PlayerSlot::Upper(Side::Right)], None),
			],
		)
	}
}
