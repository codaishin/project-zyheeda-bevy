use bevy::prelude::*;
use common::{
	tools::slot_key::SlotKey,
	traits::{
		handles_combo_menu::{
			ComboSkillDescriptor,
			GetComboAbleSkills,
			GetCombosOrdered,
			NextKeys,
		},
		handles_equipment::Combo,
		handles_loadout_menus::GetDescriptor,
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
	pub(crate) fn update(In(values): In<Option<T>>, mut commands: Commands) {
		let Some(values) = values else {
			return;
		};

		commands.insert_resource(Self(values));
	}
}

impl<T, TSkill> GetComboAbleSkills<TSkill> for EquipmentInfo<T>
where
	T: GetComboAbleSkills<TSkill>,
	TSkill: Clone,
{
	fn get_combo_able_skills(&self, key: &SlotKey) -> Vec<ComboSkillDescriptor<TSkill>> {
		self.0.get_combo_able_skills(key)
	}
}

impl<T> NextKeys for EquipmentInfo<T>
where
	T: NextKeys,
{
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey> {
		self.0.next_keys(combo_keys)
	}
}

impl<T, TSkill> GetCombosOrdered<TSkill> for EquipmentInfo<T>
where
	T: GetCombosOrdered<TSkill>,
{
	fn combos_ordered(&self) -> Vec<Combo<ComboSkillDescriptor<TSkill>>> {
		self.0.combos_ordered()
	}
}

impl<T, TKey> GetDescriptor<TKey> for EquipmentInfo<T>
where
	T: GetDescriptor<TKey>,
{
	type TItem = T::TItem;

	fn get_descriptor(&self, key: TKey) -> Option<&Self::TItem> {
		self.0.get_descriptor(key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Compatible;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_instance() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once_with(Some(_Compatible), EquipmentInfo::update)?;

		assert_eq!(
			Some(&EquipmentInfo(_Compatible)),
			app.world().get_resource::<EquipmentInfo<_Compatible>>()
		);
		Ok(())
	}
}
