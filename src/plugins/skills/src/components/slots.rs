use super::Item;
use crate::{items::slot_key::SlotKey, skills::Skill, traits::TryMap};
use bevy::ecs::component::Component;
use common::traits::accessors::get::GetRef;
use std::collections::HashMap;

#[derive(Component, Clone, PartialEq, Debug)]
pub struct Slots<TSkill = Skill>(pub HashMap<SlotKey, Option<Item<TSkill>>>);

impl<T> Slots<T> {
	pub fn new<const N: usize>(slots: [(SlotKey, Option<Item<T>>); N]) -> Self {
		Self(HashMap::from(slots))
	}
}

impl<T> Default for Slots<T> {
	fn default() -> Self {
		Self::new([])
	}
}

impl<TSkill> GetRef<SlotKey, Item<TSkill>> for Slots<TSkill> {
	fn get(&self, key: &SlotKey) -> Option<&Item<TSkill>> {
		let slot = self.0.get(key)?;
		slot.as_ref()
	}
}

impl GetRef<SlotKey, Skill> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Skill> {
		let item: &Item = self.get(key)?;

		item.skill.as_ref()
	}
}

impl<TIn, TOut> TryMap<TIn, TOut, Slots<TOut>> for Slots<TIn> {
	fn try_map(&self, map_fn: impl FnMut(&TIn) -> Option<TOut>) -> Slots<TOut> {
		let slots = self.0.iter().map(new_mapped_slot(map_fn)).collect();

		Slots(slots)
	}
}

fn new_mapped_slot<TIn, TOut>(
	mut map_fn: impl FnMut(&TIn) -> Option<TOut>,
) -> impl FnMut((&SlotKey, &Option<Item<TIn>>)) -> (SlotKey, Option<Item<TOut>>) {
	move |(key, slot)| {
		(
			*key,
			slot.as_ref().map(|slot| Item {
				name: slot.name,
				skill: slot.skill.as_ref().and_then(&mut map_fn),
				model: slot.model,
				mount: slot.mount,
				item_type: slot.item_type.clone(),
			}),
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::items::{ItemType, Mount};
	use bevy::utils::default;
	use common::components::Side;
	use std::collections::HashSet;

	#[test]
	fn get_off_hand() {
		let slots = Slots::<Skill>(
			[(
				SlotKey::BottomHand(Side::Left),
				Some(Item {
					name: "my item",
					..default()
				}),
			)]
			.into(),
		);

		assert_eq!(
			Some(&Item {
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
				Some(Item {
					name: "my item",
					..default()
				}),
			)]
			.into(),
		);

		assert_eq!(
			Some(&Item {
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
				Some(Item {
					name: "my item",
					..default()
				}),
			)]
			.into(),
		);

		assert_eq!(None::<&Item>, slots.get(&SlotKey::BottomHand(Side::Left)));
	}

	#[test]
	fn get_skill() {
		let slots = Slots::<Skill>(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(Item {
					name: "my item",
					skill: Some(Skill {
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

	#[test]
	fn try_map_item_skill() {
		#[derive(Debug, PartialEq)]
		struct _Mapped(String);

		let slots = Slots(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(Item {
					skill: Some("my/skill/path"),
					..default()
				}),
			)]
			.into(),
		);

		let got = slots.try_map(|value| Some(_Mapped(value.to_string())));
		let expected = Slots(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(Item {
					skill: Some(_Mapped("my/skill/path".to_owned())),
					..default()
				}),
			)]
			.into(),
		);

		assert_eq!(expected, got)
	}

	#[test]
	fn try_map_item_completely() {
		#[derive(Debug, PartialEq)]
		struct _Mapped(String);

		let slots = Slots(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(Item {
					name: "my item",
					skill: Some("my/skill/path"),
					model: Some("model"),
					item_type: HashSet::from([ItemType::Pistol]),
					mount: Mount::Hand,
				}),
			)]
			.into(),
		);

		let got = slots.try_map(|value| Some(_Mapped(value.to_string())));
		let expected = Slots(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(Item {
					name: "my item",
					skill: Some(_Mapped("my/skill/path".to_owned())),
					model: Some("model"),
					item_type: HashSet::from([ItemType::Pistol]),
					mount: Mount::Hand,
				}),
			)]
			.into(),
		);

		assert_eq!(expected, got)
	}

	#[test]
	fn try_map_items_without_skill() {
		#[derive(Debug, PartialEq)]
		struct _Mapped(String);

		let slots = Slots(
			[
				(
					SlotKey::BottomHand(Side::Right),
					Some(Item {
						skill: Some("my/skill/path"),
						..default()
					}),
				),
				(
					SlotKey::BottomHand(Side::Right),
					Some(Item {
						skill: None,
						..default()
					}),
				),
			]
			.into(),
		);

		let got = slots.try_map(|value| Some(_Mapped(value.to_string())));
		let expected = Slots(
			[
				(
					SlotKey::BottomHand(Side::Right),
					Some(Item {
						skill: Some(_Mapped("my/skill/path".to_owned())),
						..default()
					}),
				),
				(
					SlotKey::BottomHand(Side::Right),
					Some(Item {
						skill: None,
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
		#[derive(Debug, PartialEq)]
		struct _Mapped(String);

		let slots = Slots(
			[
				(
					SlotKey::BottomHand(Side::Right),
					Some(Item {
						skill: Some("my/skill/path"),
						..default()
					}),
				),
				(SlotKey::BottomHand(Side::Right), None),
			]
			.into(),
		);

		let got = slots.try_map(|value| Some(_Mapped(value.to_string())));
		let expected = Slots(
			[
				(
					SlotKey::BottomHand(Side::Right),
					Some(Item {
						skill: Some(_Mapped("my/skill/path".to_owned())),
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
