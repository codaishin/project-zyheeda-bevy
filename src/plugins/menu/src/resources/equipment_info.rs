use bevy::prelude::*;
use common::{
	tools::{action_key::slot::PlayerSlot, change::Change},
	traits::{
		handles_combo_menu::{Combo, GetComboAblePlayerSkills, GetCombosOrdered, NextKeys},
		handles_loadout_menu::GetItem,
		thread_safe::ThreadSafe,
	},
};
use std::collections::HashSet;

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct EquipmentInfo<T>(T);

impl<T> EquipmentInfo<T>
where
	T: ThreadSafe,
{
	pub(crate) fn update(In(values): In<Change<T>>, mut commands: Commands) {
		let Change::Some(values) = values else {
			return;
		};

		commands.insert_resource(Self(values));
	}
}

impl<T, TSkill> GetComboAblePlayerSkills<TSkill> for EquipmentInfo<T>
where
	T: GetComboAblePlayerSkills<TSkill>,
	TSkill: Clone,
{
	fn get_combo_able_player_skills(&self, key: &PlayerSlot) -> Vec<TSkill> {
		self.0.get_combo_able_player_skills(key)
	}
}

impl<T> NextKeys<PlayerSlot> for EquipmentInfo<T>
where
	T: NextKeys<PlayerSlot>,
{
	fn next_keys(&self, combo_keys: &[PlayerSlot]) -> HashSet<PlayerSlot> {
		self.0.next_keys(combo_keys)
	}
}

impl<T, TSkill> GetCombosOrdered<TSkill, PlayerSlot> for EquipmentInfo<T>
where
	T: GetCombosOrdered<TSkill, PlayerSlot>,
{
	fn combos_ordered(&self) -> Vec<Combo<PlayerSlot, TSkill>> {
		self.0.combos_ordered()
	}
}

impl<T, TKey> GetItem<TKey> for EquipmentInfo<T>
where
	T: GetItem<TKey>,
{
	type TItem = T::TItem;

	fn get_item(&self, key: TKey) -> Option<&Self::TItem> {
		self.0.get_item(key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Compatible;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_instance() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once_with(EquipmentInfo::update, Change::Some(_Compatible))?;

		assert_eq!(
			Some(&EquipmentInfo(_Compatible)),
			app.world().get_resource::<EquipmentInfo<_Compatible>>()
		);
		Ok(())
	}
}
