use crate::{skills::Skill, traits::write_item::WriteItem};
use bevy::prelude::*;
use common::tools::keys::slot::{Combo, SlotKey};

impl<T> UpdateCombos for T {}

pub(crate) trait UpdateCombos {
	fn update_for<TAgent>(
		In(updated_combos): In<Combo<Option<Skill>>>,
		mut combos: Query<&mut Self, With<TAgent>>,
	) where
		Self: Component + WriteItem<Vec<SlotKey>, Option<Skill>> + Sized,
		TAgent: Component,
	{
		let Ok(mut combos) = combos.get_single_mut() else {
			return;
		};

		for (combo_keys, skill) in updated_combos {
			combos.write_item(&combo_keys, skill);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::keys::slot::Side,
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
						eq(vec![SlotKey::TopHand(Side::Left)]),
						eq(Some(Skill {
							name: "my skill".to_owned(),
							..default()
						})),
					)
					.return_const(());
				mock.expect_write_item()
					.times(1)
					.with(eq(vec![SlotKey::TopHand(Side::Right)]), eq(None))
					.return_const(());
			}),
		));

		app.world_mut().run_system_once_with(
			vec![
				(
					vec![SlotKey::TopHand(Side::Left)],
					Some(Skill {
						name: "my skill".to_owned(),
						..default()
					}),
				),
				(vec![SlotKey::TopHand(Side::Right)], None),
			],
			_Combos::update_for::<_Agent>,
		)
	}
}
