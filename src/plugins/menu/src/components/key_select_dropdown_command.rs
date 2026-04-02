use crate::{AppendSkill, Dropdown, KeySelect, traits::GetComponent};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::{HandSlot, SlotKey},
	traits::iteration::IterFinite,
};
use std::{collections::HashSet, hash::Hash};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct KeySelectDropdownCommand<TExtra> {
	pub(crate) extra: TExtra,
	pub(crate) key_path: Vec<SlotKey>,
}

impl GetComponent for KeySelectDropdownCommand<AppendSkillCommand> {
	type TComponent = Dropdown<KeySelect<AppendSkill>>;
	type TInput = ExcludeKeys<SlotKey>;

	fn component(&self, ExcludeKeys(excluded): Self::TInput) -> Option<Self::TComponent> {
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

		Some(Dropdown { items })
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

		assert_eq!(None, command.component(ExcludeKeys(keys)));
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
			command.component(ExcludeKeys(exclude)).map(|d| d.items)
		);
	}
}
