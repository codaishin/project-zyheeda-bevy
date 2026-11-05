use crate::{AppendSkill, Dropdown, KeySelect, traits::GetComponent};
use bevy::{prelude::*, ui_widgets::Button};
use common::prelude::*;
use std::{collections::HashSet, hash::Hash};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct KeySelectDropdownCommand<TExtra> {
	pub(crate) extra: TExtra,
	pub(crate) key_path: Vec<SlotKey>,
}

impl GetComponent for KeySelectDropdownCommand<AppendSkillCommand> {
	type TBundle = (Dropdown<KeySelect<AppendSkill>>, Button);
	type TInput = ExcludeKeys<SlotKey>;

	fn get_bundle(&self, ExcludeKeys(excluded): Self::TInput) -> Option<Self::TBundle> {
		let items = HandSlot::iterator()
			.map(SlotKey::from)
			.filter(|key| !excluded.contains(key))
			.map(|on| KeySelect {
				extra: AppendSkill { on },
				key_path: self.key_path.clone(),
			})
			.collect::<Vec<_>>();

		if items.is_empty() {
			return None;
		}

		Some((Dropdown { items }, Button))
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct AppendSkillCommand;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ExcludeKeys<TKey>(pub HashSet<TKey>)
where
	TKey: Eq + Hash;

#[cfg(test)]
mod tests {
	use super::*;
	use testing::assert_eq_unordered;

	#[test]
	fn get_no_dropdown_when_all_keys_excluded() {
		let keys = HandSlot::iterator().map(SlotKey::from).collect();
		let command = KeySelectDropdownCommand {
			extra: AppendSkillCommand,
			key_path: vec![SlotKey::from(HandSlot::Left)],
		};

		assert!(command.get_bundle(ExcludeKeys(keys)).is_none());
	}

	#[test]
	fn get_dropdown_with_remaining_keys() {
		let exclude = HandSlot::iterator()
			.filter(|k| k != &HandSlot::Left)
			.map(SlotKey::from)
			.collect();
		let command = KeySelectDropdownCommand {
			extra: AppendSkillCommand,
			key_path: vec![SlotKey::from(HandSlot::Right)],
		};

		assert_eq_unordered!(
			Some(vec![KeySelect {
				extra: AppendSkill {
					on: SlotKey::from(HandSlot::Left)
				},
				key_path: vec![SlotKey::from(HandSlot::Right)]
			}]),
			command
				.get_bundle(ExcludeKeys(exclude))
				.map(|(d, ..)| d.items)
		);
	}
}
