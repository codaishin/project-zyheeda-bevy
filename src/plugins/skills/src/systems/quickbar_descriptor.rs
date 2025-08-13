use std::collections::HashMap;

use crate::{
	item::Item,
	skills::{QueuedSkill, Skill},
	tools::{cache::Cache, quickbar_item::QuickbarItem},
	traits::peek_next::PeekNext,
};
use bevy::prelude::*;
use common::{
	tools::{
		action_key::slot::{PlayerSlot, SlotKey},
		change::Change,
		item_type::ItemType,
		skill_execution::SkillExecution,
	},
	traits::iterate::Iterate,
};

#[allow(clippy::type_complexity)]
pub(crate) fn get_quickbar_descriptors_for<TAgent, TSlots, TQueue, TCombos>(
	queues: Query<(Ref<TSlots>, Ref<TQueue>, Ref<TCombos>), With<TAgent>>,
	items: Res<Assets<Item>>,
	skills: Res<Assets<Skill>>,
) -> Change<Cache<PlayerSlot, QuickbarItem>>
where
	TAgent: Component,
	for<'a> TSlots: Component + Iterate<'a, TItem = (SlotKey, &'a Option<Handle<Item>>)>,
	for<'a> TQueue: Component + Iterate<'a, TItem = &'a QueuedSkill>,
	for<'a> TCombos: Component + PeekNext<'a, TNext = Skill>,
{
	let Ok((slots, queue, combos)) = queues.single() else {
		return Change::None;
	};

	if !any_true(&[slots.is_changed(), queue.is_changed(), combos.is_changed()]) {
		return Change::None;
	}

	let combos = combos.as_ref();
	let mut queue = queue.iterate();
	let active = queue.next().map(get_key_and_skill);
	let queued = queue.map(get_key_and_skill);
	let queued = collect_without_duplicates(queued);

	let map = slots
		.iterate()
		.filter_map(|(key, handle)| Some((key, PlayerSlot::try_from(key).ok()?, handle)))
		.filter_map(|(key, player_key, handle)| {
			let (item, inactive) = get_assets(&items, &skills, handle)?;
			let (mut skill, execution) = select_skill(key, inactive, &active, &queued);
			set_combo_for_inactive(&mut skill, execution, combos, key, &item.item_type);

			Some((
				player_key,
				QuickbarItem {
					skill_token: skill.token.clone(),
					skill_icon: skill.icon.clone(),
					execution,
				},
			))
		})
		.collect();

	Change::Some(Cache(map))
}

fn get_key_and_skill(skill: &QueuedSkill) -> (SlotKey, &'_ Skill) {
	(skill.key, &skill.skill)
}

fn collect_without_duplicates<'a>(
	items: impl Iterator<Item = (SlotKey, &'a Skill)>,
) -> HashMap<SlotKey, &'a Skill> {
	let mut map = HashMap::default();

	for (key, item) in items {
		if map.contains_key(&key) {
			continue;
		}
		map.insert(key, item);
	}

	map
}

fn select_skill<'a>(
	slot_key: SlotKey,
	inactive: &'a Skill,
	active: &Option<(SlotKey, &'a Skill)>,
	queued: &HashMap<SlotKey, &'a Skill>,
) -> (&'a Skill, SkillExecution) {
	match active {
		Some((key, queued)) if key == &slot_key => return (queued, SkillExecution::Active),
		_ => {}
	}

	if let Some(queued) = queued.get(&slot_key) {
		return (*queued, SkillExecution::Queued);
	}

	(inactive, SkillExecution::None)
}

fn set_combo_for_inactive<'a, TCombos>(
	skill: &mut &'a Skill,
	execution: SkillExecution,
	combos: &'a TCombos,
	key: SlotKey,
	item_type: &ItemType,
) where
	TCombos: PeekNext<'a, TNext = Skill>,
{
	if execution != SkillExecution::None {
		return;
	}
	let Some(combo) = combos.peek_next(key, item_type) else {
		return;
	};

	*skill = combo
}

fn get_assets<'a>(
	items: &'a Assets<Item>,
	skills: &'a Assets<Skill>,
	handle: &'a Option<Handle<Item>>,
) -> Option<(&'a Item, &'a Skill)> {
	let handle = handle.as_ref()?;
	let item = items.get(handle)?;
	let handle = item.skill.as_ref()?;
	let skill = skills.get(handle)?;
	Some((item, skill))
}

fn any_true(values: &[bool]) -> bool {
	values.contains(&true)
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::{action_key::slot::Side, item_type::ItemType},
		traits::handles_localization::Token,
	};
	use std::{array::IntoIter, collections::HashMap, slice::Iter};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component)]
	struct _Slots(SlotKey, Option<Handle<Item>>);

	impl _Slots {
		fn new(
			app: &mut App,
			slot_key: SlotKey,
			item_type: ItemType,
			skill_token: &'static str,
			skill_icon: Option<Handle<Image>>,
		) -> Self {
			let skill = app.world_mut().resource_mut::<Assets<Skill>>().add(Skill {
				token: Token::from(skill_token),
				icon: skill_icon,
				..default()
			});
			let item = app.world_mut().resource_mut::<Assets<Item>>().add(Item {
				item_type,
				skill: Some(skill),
				..default()
			});

			Self(slot_key, Some(item))
		}
	}

	impl<'a> Iterate<'a> for _Slots {
		type TItem = (SlotKey, &'a Option<Handle<Item>>);
		type TIter = IntoIter<(SlotKey, &'a Option<Handle<Item>>), 1>;

		fn iterate(&'a self) -> Self::TIter {
			let _Slots(key, slot) = self;
			[(*key, slot)].into_iter()
		}
	}

	#[derive(Component, Default)]
	struct _Queue(Vec<QueuedSkill>);

	impl<'a> Iterate<'a> for _Queue {
		type TItem = &'a QueuedSkill;
		type TIter = Iter<'a, QueuedSkill>;

		fn iterate(&'a self) -> Self::TIter {
			self.0.iter()
		}
	}

	#[derive(Component, Default)]
	struct _NextComboSkills(HashMap<(SlotKey, ItemType), Skill>);

	impl<'a> PeekNext<'a> for _NextComboSkills {
		type TNext = Skill;

		fn peek_next(&'a self, trigger: SlotKey, item_type: &ItemType) -> Option<&'a Self::TNext> {
			self.0.get(&(trigger, *item_type))
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Change<Cache<PlayerSlot, QuickbarItem>>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Assets<Skill>>();
		app.init_resource::<Assets<Item>>();
		app.add_systems(
			Update,
			get_quickbar_descriptors_for::<_Agent, _Slots, _Queue, _NextComboSkills>.pipe(
				|In(c), mut commands: Commands| {
					commands.insert_resource(_Result(c));
				},
			),
		);

		app
	}

	#[test]
	fn return_inactive() {
		let mut app = setup();
		let icon = Some(new_handle());
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			icon.clone(),
		);
		app.world_mut().spawn((
			_Agent,
			slots,
			_Queue::default(),
			_NextComboSkills::default(),
		));

		app.update();

		assert_eq!(
			Some(&_Result(Change::Some(Cache(HashMap::from([(
				PlayerSlot::Upper(Side::Left),
				QuickbarItem {
					skill_token: Token::from("my skill"),
					skill_icon: icon,
					execution: SkillExecution::None,
				}
			)]))))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn return_active() {
		let mut app = setup();
		let icon = Some(new_handle());
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			Some(new_handle()),
		);
		app.world_mut().spawn((
			_Agent,
			slots,
			_Queue(vec![QueuedSkill {
				key: SlotKey::from(PlayerSlot::Upper(Side::Left)),
				skill: Skill {
					token: Token::from("my active skill"),
					icon: icon.clone(),
					..default()
				},
				..default()
			}]),
			_NextComboSkills::default(),
		));

		app.update();

		assert_eq!(
			Some(&_Result(Change::Some(Cache(HashMap::from([(
				PlayerSlot::Upper(Side::Left),
				QuickbarItem {
					skill_token: Token::from("my active skill"),
					skill_icon: icon,
					execution: SkillExecution::Active,
				}
			)]))))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn return_queued() {
		let mut app = setup();
		let icon = Some(new_handle());
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			Some(new_handle()),
		);
		app.world_mut().spawn((
			_Agent,
			slots,
			_Queue(vec![
				QueuedSkill {
					key: SlotKey::from(PlayerSlot::Upper(Side::Right)),
					..default()
				},
				QueuedSkill {
					key: SlotKey::from(PlayerSlot::Upper(Side::Left)),
					skill: Skill {
						token: Token::from("my queued skill"),
						icon: icon.clone(),
						..default()
					},
					..default()
				},
			]),
			_NextComboSkills::default(),
		));

		app.update();

		assert_eq!(
			Some(&_Result(Change::Some(Cache(HashMap::from([(
				PlayerSlot::Upper(Side::Left),
				QuickbarItem {
					skill_token: Token::from("my queued skill"),
					skill_icon: icon,
					execution: SkillExecution::Queued,
				}
			)]))))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn return_first_queued() {
		let mut app = setup();
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			None,
		);
		app.world_mut().spawn((
			_Agent,
			slots,
			_Queue(vec![
				QueuedSkill {
					key: SlotKey::from(PlayerSlot::Upper(Side::Right)),
					..default()
				},
				QueuedSkill {
					key: SlotKey::from(PlayerSlot::Upper(Side::Left)),
					skill: Skill {
						token: Token::from("my queued skill"),
						..default()
					},
					..default()
				},
				QueuedSkill {
					key: SlotKey::from(PlayerSlot::Upper(Side::Left)),
					skill: Skill {
						token: Token::from("my other queued skill"),
						..default()
					},
					..default()
				},
			]),
			_NextComboSkills::default(),
		));

		app.update();

		assert_eq!(
			Some(&_Result(Change::Some(Cache(HashMap::from([(
				PlayerSlot::Upper(Side::Left),
				QuickbarItem {
					skill_token: Token::from("my queued skill"),
					execution: SkillExecution::Queued,
					..default()
				}
			)]))))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn return_inactive_combo_skill() {
		let mut app = setup();
		let icon = Some(new_handle());
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			Some(new_handle()),
		);
		app.world_mut().spawn((
			_Agent,
			slots,
			_Queue::default(),
			_NextComboSkills(HashMap::from([(
				(
					SlotKey::from(PlayerSlot::Upper(Side::Left)),
					ItemType::Bracer,
				),
				Skill {
					token: Token::from("my combo skill"),
					icon: icon.clone(),
					..default()
				},
			)])),
		));

		app.update();

		assert_eq!(
			Some(&_Result(Change::Some(Cache(HashMap::from([(
				PlayerSlot::Upper(Side::Left),
				QuickbarItem {
					skill_token: Token::from("my combo skill"),
					skill_icon: icon,
					execution: SkillExecution::None,
				}
			)]))))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn do_not_return_combo_skill_for_active() {
		let mut app = setup();
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			None,
		);
		app.world_mut().spawn((
			_Agent,
			slots,
			_Queue(vec![QueuedSkill {
				key: SlotKey::from(PlayerSlot::Upper(Side::Left)),
				skill: Skill {
					token: Token::from("my active skill"),
					..default()
				},
				..default()
			}]),
			_NextComboSkills(HashMap::from([(
				(
					SlotKey::from(PlayerSlot::Upper(Side::Left)),
					ItemType::Bracer,
				),
				Skill {
					token: Token::from("my combo skill"),
					..default()
				},
			)])),
		));

		app.update();

		assert_eq!(
			Some(&_Result(Change::Some(Cache(HashMap::from([(
				PlayerSlot::Upper(Side::Left),
				QuickbarItem {
					skill_token: Token::from("my active skill"),
					execution: SkillExecution::Active,
					..default()
				}
			)]))))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn do_not_return_combo_skill_for_queued() {
		let mut app = setup();
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			None,
		);
		app.world_mut().spawn((
			_Agent,
			slots,
			_Queue(vec![
				QueuedSkill {
					key: SlotKey::from(PlayerSlot::Upper(Side::Right)),
					skill: Skill {
						token: Token::from("my active skill"),
						..default()
					},
					..default()
				},
				QueuedSkill {
					key: SlotKey::from(PlayerSlot::Upper(Side::Left)),
					skill: Skill {
						token: Token::from("my queued skill"),
						..default()
					},
					..default()
				},
			]),
			_NextComboSkills(HashMap::from([(
				(
					SlotKey::from(PlayerSlot::Upper(Side::Left)),
					ItemType::Bracer,
				),
				Skill {
					token: Token::from("my combo skill"),
					..default()
				},
			)])),
		));

		app.update();

		assert_eq!(
			Some(&_Result(Change::Some(Cache(HashMap::from([(
				PlayerSlot::Upper(Side::Left),
				QuickbarItem {
					skill_token: Token::from("my queued skill"),
					execution: SkillExecution::Queued,
					..default()
				}
			)]))))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn return_none_when_components_did_not_change() {
		let mut app = setup();
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			None,
		);
		app.world_mut().spawn((
			_Agent,
			slots,
			_Queue::default(),
			_NextComboSkills::default(),
		));

		app.update();
		app.update();

		assert_eq!(
			Some(&_Result(Change::None)),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn return_some_when_slots_changed() {
		let mut app = setup();
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			None,
		);
		let agent = app
			.world_mut()
			.spawn((
				_Agent,
				slots,
				_Queue::default(),
				_NextComboSkills::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_Slots>()
			.as_deref_mut();
		app.update();

		assert!(matches!(
			app.world().get_resource::<_Result>(),
			Some(&_Result(Change::Some(_)))
		));
	}

	#[test]
	fn return_some_when_queue_changed() {
		let mut app = setup();
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			None,
		);
		let agent = app
			.world_mut()
			.spawn((
				_Agent,
				slots,
				_Queue::default(),
				_NextComboSkills::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_Queue>()
			.as_deref_mut();
		app.update();

		assert!(matches!(
			app.world().get_resource::<_Result>(),
			Some(&_Result(Change::Some(_)))
		));
	}

	#[test]
	fn return_some_when_combos_changed() {
		let mut app = setup();
		let slots = _Slots::new(
			&mut app,
			SlotKey::from(PlayerSlot::Upper(Side::Left)),
			ItemType::Bracer,
			"my skill",
			None,
		);
		let agent = app
			.world_mut()
			.spawn((
				_Agent,
				slots,
				_Queue::default(),
				_NextComboSkills::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_NextComboSkills>()
			.as_deref_mut();
		app.update();

		assert!(matches!(
			app.world().get_resource::<_Result>(),
			Some(&_Result(Change::Some(_)))
		));
	}
}
