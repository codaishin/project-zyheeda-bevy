use crate::{item::SkillItem, skills::Skill, slot_key::SlotKey, traits::TryMap};
use bevy::ecs::component::Component;
use common::traits::accessors::get::GetRef;
use items::traits::item_type::AssociatedItemType;
use std::{collections::HashMap, fmt::Debug};

#[derive(Component, Clone, PartialEq, Debug)]
pub struct Slots<TSkill = Skill>(pub HashMap<SlotKey, Option<SkillItem<TSkill>>>)
where
	TSkill: AssociatedItemType,
	TSkill::TItemType: Clone + PartialEq + Debug;

impl<T> Slots<T>
where
	T: AssociatedItemType,
	T::TItemType: Clone + PartialEq + Debug,
{
	pub fn new<const N: usize>(slots: [(SlotKey, Option<SkillItem<T>>); N]) -> Self {
		Self(HashMap::from(slots))
	}
}

impl<T> Default for Slots<T>
where
	T: AssociatedItemType,
	T::TItemType: Clone + PartialEq + Debug,
{
	fn default() -> Self {
		Self::new([])
	}
}

impl<T> GetRef<SlotKey, SkillItem<T>> for Slots<T>
where
	T: AssociatedItemType,
	T::TItemType: Clone + PartialEq + Debug,
{
	fn get(&self, key: &SlotKey) -> Option<&SkillItem<T>> {
		let slot = self.0.get(key)?;
		slot.as_ref()
	}
}

impl GetRef<SlotKey, Skill> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Skill> {
		let item: &SkillItem = self.get(key)?;

		item.content.as_ref()
	}
}

impl<TIn, TOut> TryMap<TIn, TOut, Slots<TOut>> for Slots<TIn>
where
	TIn: AssociatedItemType,
	TIn::TItemType: Clone + PartialEq + Debug,
	TOut: AssociatedItemType,
	TOut::TItemType: Clone + PartialEq + Debug,
	TOut::TItemType: From<TIn::TItemType>,
{
	fn try_map(&self, map_fn: impl FnMut(&TIn) -> Option<TOut>) -> Slots<TOut> {
		let slots = self.0.iter().map(new_mapped_slot(map_fn)).collect();

		Slots(slots)
	}
}

fn new_mapped_slot<TIn, TOut>(
	mut map_fn: impl FnMut(&TIn) -> Option<TOut>,
) -> impl FnMut((&SlotKey, &Option<SkillItem<TIn>>)) -> (SlotKey, Option<SkillItem<TOut>>)
where
	TIn: AssociatedItemType,
	TIn::TItemType: Clone + PartialEq + Debug,
	TOut: AssociatedItemType,
	TOut::TItemType: Clone + PartialEq + Debug,
	TOut::TItemType: From<TIn::TItemType>,
{
	move |(key, slot)| {
		(
			*key,
			slot.as_ref().map(|slot| SkillItem {
				name: slot.name,
				content: slot.content.as_ref().and_then(&mut map_fn),
				model: slot.model,
				item_type: slot.item_type.clone().into(),
			}),
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::components::Side;

	#[test]
	fn get_off_hand() {
		let slots = Slots::<Skill>(
			[(
				SlotKey::BottomHand(Side::Left),
				Some(SkillItem {
					name: "my item",
					..default()
				}),
			)]
			.into(),
		);

		assert_eq!(
			Some(&SkillItem {
				name: "my item",
				..default()
			}),
			slots.get(&SlotKey::BottomHand(Side::Left))
		);
	}

	#[test]
	fn get_main_hand() {
		let slots = Slots::<Skill>(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem {
					name: "my item",
					..default()
				}),
			)]
			.into(),
		);

		assert_eq!(
			Some(&SkillItem {
				name: "my item",
				..default()
			}),
			slots.get(&SlotKey::BottomHand(Side::Right))
		);
	}

	#[test]
	fn get_none() {
		let slots = Slots::<Skill>(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem {
					name: "my item",
					..default()
				}),
			)]
			.into(),
		);

		assert_eq!(
			None::<&SkillItem>,
			slots.get(&SlotKey::BottomHand(Side::Left))
		);
	}

	#[test]
	fn get_skill() {
		let slots = Slots::<Skill>(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem {
					name: "my item",
					content: Some(Skill {
						name: "my skill".to_owned(),
						..default()
					}),
					..default()
				}),
			)]
			.into(),
		);

		assert_eq!(
			Some(&Skill {
				name: "my skill".to_owned(),
				..default()
			}),
			slots.get(&SlotKey::BottomHand(Side::Right))
		);
	}

	#[derive(Debug, PartialEq, Clone)]
	struct _UnMapped(&'static str);

	#[derive(Debug, PartialEq, Clone, Default)]
	struct _ItemTypeUnmapped;

	impl AssociatedItemType for _UnMapped {
		type TItemType = _ItemTypeUnmapped;
	}

	#[derive(Debug, PartialEq, Clone)]
	struct _Mapped(String);

	#[derive(Debug, PartialEq, Clone, Default)]
	struct _ItemTypeMapped;

	impl AssociatedItemType for _Mapped {
		type TItemType = _ItemTypeMapped;
	}

	impl From<_ItemTypeUnmapped> for _ItemTypeMapped {
		fn from(_: _ItemTypeUnmapped) -> Self {
			_ItemTypeMapped
		}
	}

	#[test]
	fn try_map_item_skill() {
		let slots = Slots(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem {
					content: Some(_UnMapped("my/skill/path")),
					..default()
				}),
			)]
			.into(),
		);

		let got = slots.try_map(|_UnMapped(value)| Some(_Mapped(value.to_string())));
		let expected = Slots(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem {
					content: Some(_Mapped("my/skill/path".to_owned())),
					..default()
				}),
			)]
			.into(),
		);

		assert_eq!(expected, got)
	}

	#[test]
	fn try_map_items_without_skill() {
		let slots = Slots(
			[
				(
					SlotKey::BottomHand(Side::Right),
					Some(SkillItem {
						content: Some(_UnMapped("my/skill/path")),
						..default()
					}),
				),
				(
					SlotKey::BottomHand(Side::Right),
					Some(SkillItem {
						content: None,
						..default()
					}),
				),
			]
			.into(),
		);

		let got = slots.try_map(|_UnMapped(value)| Some(_Mapped(value.to_string())));
		let expected = Slots(
			[
				(
					SlotKey::BottomHand(Side::Right),
					Some(SkillItem {
						content: Some(_Mapped("my/skill/path".to_owned())),
						..default()
					}),
				),
				(
					SlotKey::BottomHand(Side::Right),
					Some(SkillItem {
						content: None,
						..default()
					}),
				),
			]
			.into(),
		);

		assert_eq!(expected, got)
	}

	#[test]
	fn try_map_slots_without_items() {
		let slots = Slots(
			[
				(
					SlotKey::BottomHand(Side::Right),
					Some(SkillItem {
						content: Some(_UnMapped("my/skill/path")),
						..default()
					}),
				),
				(SlotKey::BottomHand(Side::Right), None),
			]
			.into(),
		);

		let got = slots.try_map(|_UnMapped(value)| Some(_Mapped(value.to_string())));
		let expected = Slots(
			[
				(
					SlotKey::BottomHand(Side::Right),
					Some(SkillItem {
						content: Some(_Mapped("my/skill/path".to_owned())),
						..default()
					}),
				),
				(SlotKey::BottomHand(Side::Right), None),
			]
			.into(),
		);

		assert_eq!(expected, got)
	}
}
