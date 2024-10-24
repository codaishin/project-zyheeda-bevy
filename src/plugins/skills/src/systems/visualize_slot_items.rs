use crate::{components::slots::Slots, item::SkillItemContent, slot_key::SlotKey};
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;
use items::{
	components::visualize::VisualizeCommands,
	traits::{key_string::KeyString, uses_view::UsesView},
};

#[allow(clippy::type_complexity)]
pub(crate) fn visualize_slot_items<TView>(
	mut commands: Commands,
	agents: Query<(Entity, &Slots), Changed<Slots>>,
) where
	TView: KeyString<SlotKey> + Sync + Send + 'static,
	SkillItemContent: UsesView<TView>,
{
	for (entity, slots) in &agents {
		let mut visualize = VisualizeCommands::<TView>::default();

		for (key, item) in &slots.0 {
			visualize = visualize.with_item(key, item.as_ref());
		}

		commands.try_insert_on(entity, visualize);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		item::{item_type::SkillItemType, SkillItem, SkillItemContent},
		skills::Skill,
		slot_key::SlotKey,
	};
	use bevy::ecs::system::RunSystemOnce;
	use common::components::{AssetModel, Side};
	use items::{components::visualize::VisualizeCommands, traits::key_string::KeyString};

	#[derive(Debug, PartialEq)]
	struct _View;

	impl KeyString<SlotKey> for _View {
		fn key_string(key: &SlotKey) -> &'static str {
			match key {
				SlotKey::TopHand(_) => "top",
				SlotKey::BottomHand(_) => "btm",
			}
		}
	}

	impl UsesView<_View> for SkillItemContent {
		fn uses_view(&self) -> bool {
			true
		}
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn visualize_item() {
		let mut app = setup();
		let item = SkillItem {
			content: SkillItemContent {
				model: AssetModel::Path("my model"),
				..default()
			},
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
			.run_system_once(visualize_slot_items::<_View>);

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(
				&VisualizeCommands::<_View>::default()
					.with_item(&SlotKey::BottomHand(Side::Right), Some(&item))
			),
			entity.get::<VisualizeCommands<_View>>(),
		);
	}

	#[test]
	fn visualize_items() {
		let mut app = setup();
		let item_a = SkillItem {
			content: SkillItemContent {
				model: AssetModel::Path("my bracer model"),
				item_type: SkillItemType::Pistol,
				..default()
			},
			..default()
		};
		let item_b = SkillItem {
			content: SkillItemContent {
				model: AssetModel::Path("my forearm model"),
				item_type: SkillItemType::Bracer,
				..default()
			},
			..default()
		};
		let entity = app
			.world_mut()
			.spawn(Slots::<Skill>::new([
				(SlotKey::BottomHand(Side::Right), Some(item_a.clone())),
				(SlotKey::TopHand(Side::Right), Some(item_b.clone())),
			]))
			.id();

		app.world_mut()
			.run_system_once(visualize_slot_items::<_View>);

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(
				&VisualizeCommands::<_View>::default()
					.with_item(&SlotKey::BottomHand(Side::Right), Some(&item_a))
					.with_item(&SlotKey::TopHand(Side::Right), Some(&item_b))
			),
			entity.get::<VisualizeCommands<_View>>(),
		);
	}

	#[test]
	fn visualize_item_only_once() {
		let mut app = setup();
		let item = SkillItem {
			content: SkillItemContent {
				model: AssetModel::Path("my model"),
				..default()
			},
			..default()
		};
		let entity = app
			.world_mut()
			.spawn(Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Right),
				Some(item.clone()),
			)]))
			.id();

		app.add_systems(Update, visualize_slot_items::<_View>);
		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<VisualizeCommands<_View>>();
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(None, entity.get::<VisualizeCommands<_View>>());
	}

	#[test]
	fn visualize_items_again_after_slots_mutably_dereferenced() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem {
					content: SkillItemContent {
						model: AssetModel::Path("my model"),
						..default()
					},
					..default()
				}),
			)]))
			.id();

		app.add_systems(Update, visualize_slot_items::<_View>);
		app.update();
		let mut agent = app.world_mut().entity_mut(entity);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		let item = SkillItem {
			content: SkillItemContent {
				model: AssetModel::Path("my other model"),
				..default()
			},
			..default()
		};
		*slots = Slots::<Skill>::new([(SlotKey::TopHand(Side::Right), Some(item.clone()))]);
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(
				&VisualizeCommands::<_View>::default()
					.with_item(&SlotKey::TopHand(Side::Right), Some(&item))
			),
			entity.get::<VisualizeCommands<_View>>(),
		);
	}
}
