use crate::{
	components::slots::Slots,
	definitions::item_slots::{ForearmSlots, HandSlots},
	slot_key::SlotKey,
};
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;
use items::{components::visualize::Visualize, traits::key_string::KeyString};

#[allow(clippy::type_complexity)]
pub(crate) fn visualize_slot_items<TAgent>(
	mut commands: Commands,
	agents: Query<(Entity, &Slots), (With<TAgent>, Changed<Slots>)>,
) where
	TAgent: Component,
	HandSlots<TAgent>: KeyString<SlotKey>,
	ForearmSlots<TAgent>: KeyString<SlotKey>,
{
	for (entity, slots) in &agents {
		let mut hand_slots = Visualize::<HandSlots<TAgent>>::default();
		let mut forearm_slots = Visualize::<ForearmSlots<TAgent>>::default();

		for (key, item) in &slots.0 {
			hand_slots = hand_slots.with_item(key, item.as_ref());
			forearm_slots = forearm_slots.with_item(key, item.as_ref());
		}

		commands.try_insert_on(entity, (hand_slots, forearm_slots));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		definitions::item_slots::{ForearmSlots, HandSlots},
		item::{item_type::SkillItemType, SkillItem},
		skills::Skill,
		slot_key::SlotKey,
	};
	use bevy::{app::App, ecs::system::RunSystemOnce};
	use common::components::Side;
	use items::{components::visualize::Visualize, traits::key_string::KeyString};

	#[derive(Component, Debug, PartialEq)]
	struct _Agent;

	impl KeyString<SlotKey> for HandSlots<_Agent> {
		fn key_string(key: &SlotKey) -> &'static str {
			match key {
				SlotKey::TopHand(_) => "top",
				SlotKey::BottomHand(_) => "btm",
			}
		}
	}

	impl KeyString<SlotKey> for ForearmSlots<_Agent> {
		fn key_string(key: &SlotKey) -> &'static str {
			match key {
				SlotKey::TopHand(_) => "top",
				SlotKey::BottomHand(_) => "btm",
			}
		}
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn visualize_item() {
		let mut app = setup();
		let item = SkillItem {
			model: Some("my model"),
			..default()
		};
		let entity = app
			.world_mut()
			.spawn((
				_Agent,
				Slots::<Skill>::new([(SlotKey::BottomHand(Side::Right), Some(item.clone()))]),
			))
			.id();

		app.world_mut()
			.run_system_once(visualize_slot_items::<_Agent>);

		let entity = app.world().entity(entity);
		assert_eq!(
			(
				Some(
					&Visualize::<HandSlots<_Agent>>::default()
						.with_item(&SlotKey::BottomHand(Side::Right), Some(&item))
				),
				Some(
					&Visualize::<ForearmSlots<_Agent>>::default()
						.with_item(&SlotKey::BottomHand(Side::Right), Some(&item))
				)
			),
			(
				entity.get::<Visualize<HandSlots<_Agent>>>(),
				entity.get::<Visualize<ForearmSlots<_Agent>>>()
			),
		)
	}

	#[test]
	fn visualize_items() {
		let mut app = setup();
		let item_a = SkillItem {
			model: Some("my bracer model"),
			item_type: SkillItemType::Pistol,
			..default()
		};
		let item_b = SkillItem {
			model: Some("my forearm model"),
			item_type: SkillItemType::Bracer,
			..default()
		};
		let entity = app
			.world_mut()
			.spawn((
				_Agent,
				Slots::<Skill>::new([
					(SlotKey::BottomHand(Side::Right), Some(item_a.clone())),
					(SlotKey::TopHand(Side::Right), Some(item_b.clone())),
				]),
			))
			.id();

		app.world_mut()
			.run_system_once(visualize_slot_items::<_Agent>);

		let entity = app.world().entity(entity);
		assert_eq!(
			(
				Some(
					&Visualize::<HandSlots<_Agent>>::default()
						.with_item(&SlotKey::BottomHand(Side::Right), Some(&item_a))
						.with_item(&SlotKey::TopHand(Side::Right), Some(&item_b))
				),
				Some(
					&Visualize::<ForearmSlots<_Agent>>::default()
						.with_item(&SlotKey::BottomHand(Side::Right), Some(&item_a))
						.with_item(&SlotKey::TopHand(Side::Right), Some(&item_b))
				)
			),
			(
				entity.get::<Visualize<HandSlots<_Agent>>>(),
				entity.get::<Visualize<ForearmSlots<_Agent>>>()
			),
		)
	}

	#[test]
	fn do_nothing_when_not_with_agent_component() {
		let mut app = setup();
		let item = SkillItem {
			model: Some("my model"),
			..default()
		};
		let entity = app
			.world_mut()
			.spawn(Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Right),
				Some(item.clone()),
			)]))
			.id();

		app.world_mut()
			.run_system_once(visualize_slot_items::<_Agent>);

		let entity = app.world().entity(entity);
		assert_eq!(
			(None, None),
			(
				entity.get::<Visualize<HandSlots<_Agent>>>(),
				entity.get::<Visualize<ForearmSlots<_Agent>>>()
			),
		)
	}

	#[test]
	fn visualize_item_only_once() {
		let mut app = setup();
		let item = SkillItem {
			model: Some("my model"),
			..default()
		};
		let entity = app
			.world_mut()
			.spawn((
				_Agent,
				Slots::<Skill>::new([(SlotKey::BottomHand(Side::Right), Some(item.clone()))]),
			))
			.id();

		app.add_systems(Update, visualize_slot_items::<_Agent>);
		app.update();
		app.world_mut().entity_mut(entity).remove::<(
			Visualize<HandSlots<_Agent>>,
			Visualize<ForearmSlots<_Agent>>,
		)>();
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			(None, None),
			(
				entity.get::<Visualize<HandSlots<_Agent>>>(),
				entity.get::<Visualize<ForearmSlots<_Agent>>>()
			),
		)
	}

	#[test]
	fn visualize_items_again_after_slots_mutably_dereferenced() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Agent,
				Slots::<Skill>::new([(
					SlotKey::BottomHand(Side::Right),
					Some(SkillItem {
						model: Some("my model"),
						..default()
					}),
				)]),
			))
			.id();

		app.add_systems(Update, visualize_slot_items::<_Agent>);
		app.update();
		let mut agent = app.world_mut().entity_mut(entity);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		let item = SkillItem {
			model: Some("my other model"),
			..default()
		};
		*slots = Slots::<Skill>::new([(SlotKey::TopHand(Side::Right), Some(item.clone()))]);
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			(
				Some(
					&Visualize::<HandSlots<_Agent>>::default()
						.with_item(&SlotKey::TopHand(Side::Right), Some(&item))
				),
				Some(
					&Visualize::<ForearmSlots<_Agent>>::default()
						.with_item(&SlotKey::TopHand(Side::Right), Some(&item))
				)
			),
			(
				entity.get::<Visualize<HandSlots<_Agent>>>(),
				entity.get::<Visualize<ForearmSlots<_Agent>>>()
			),
		)
	}
}
