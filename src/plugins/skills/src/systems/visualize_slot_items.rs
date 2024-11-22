use crate::{
	components::slots::Slots,
	item::{SkillItem, SkillItemContent},
	slot_key::SlotKey,
};
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;
use items::{
	components::visualize::VisualizeCommands,
	traits::{get_view_data::GetViewData, view::ItemView},
};

#[allow(clippy::type_complexity)]
pub(crate) fn visualize_slot_items<TView>(
	mut commands: Commands,
	agents: Query<(Entity, &Slots), Changed<Slots>>,
	items: Res<Assets<SkillItem>>,
) where
	TView: ItemView<SlotKey> + Sync + Send + 'static,
	SkillItemContent: GetViewData<TView, SlotKey>,
{
	for (entity, slots) in &agents {
		let mut visualize = VisualizeCommands::<TView, SlotKey>::default();

		for (key, item) in &slots.0 {
			let item = item.as_ref().and_then(|item| items.get(item));
			visualize = visualize.with_item(key, item);
		}

		commands.try_insert_on(entity, visualize);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::model_render::ModelRender,
		item::{SkillItem, SkillItemContent},
		slot_key::SlotKey,
	};
	use bevy::ecs::system::RunSystemOnce;
	use common::{
		components::{AssetModel, Side},
		test_tools::utils::new_handle,
	};
	use items::components::visualize::VisualizeCommands;
	use std::ops::DerefMut;

	#[derive(Debug, PartialEq)]
	struct _View;

	#[derive(Component, Debug, PartialEq, Default, Clone)]
	struct _ViewComponent(SkillItemContent);

	impl ItemView<SlotKey> for _View {
		type TFilter = ();
		type TViewComponents = _ViewComponent;

		fn view_entity_name(key: &SlotKey) -> &'static str {
			match key {
				SlotKey::TopHand(_) => "top",
				SlotKey::BottomHand(_) => "btm",
			}
		}
	}

	impl GetViewData<_View, SlotKey> for SkillItemContent {
		fn get_view_data(&self) -> _ViewComponent {
			_ViewComponent(self.clone())
		}
	}

	fn setup<const N: usize>(items: [(AssetId<SkillItem>, SkillItem); N]) -> App {
		let mut app = App::new();
		let mut item_assets = Assets::default();

		for (id, item) in items {
			item_assets.insert(id, item);
		}

		app.insert_resource(item_assets);
		app
	}

	#[test]
	fn visualize_item() {
		let handle = new_handle();
		let item = SkillItem {
			content: SkillItemContent {
				model: ModelRender::Hand(AssetModel::path("my model")),
				..default()
			},
			..default()
		};
		let mut app = setup([(handle.id(), item.clone())]);
		let entity = app
			.world_mut()
			.spawn(Slots::new([(
				SlotKey::BottomHand(Side::Right),
				Some(handle),
			)]))
			.id();

		app.world_mut()
			.run_system_once(visualize_slot_items::<_View>);

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(
				&VisualizeCommands::<_View, SlotKey>::default()
					.with_item(&SlotKey::BottomHand(Side::Right), Some(&item))
			),
			entity.get::<VisualizeCommands<_View, SlotKey>>(),
		);
	}

	#[test]
	fn visualize_none_if_handle_is_none() {
		let mut app = setup([]);
		let entity = app
			.world_mut()
			.spawn(Slots::new([(SlotKey::BottomHand(Side::Right), None)]))
			.id();

		app.world_mut()
			.run_system_once(visualize_slot_items::<_View>);

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&VisualizeCommands::<_View, SlotKey>::default().with_item(
				&SlotKey::BottomHand(Side::Right),
				None as Option<&SkillItem>
			)),
			entity.get::<VisualizeCommands<_View, SlotKey>>(),
		);
	}

	#[test]
	fn visualize_none_if_item_cannot_be_found() {
		let mut app = setup([]);
		let entity = app
			.world_mut()
			.spawn(Slots::new([(
				SlotKey::BottomHand(Side::Right),
				Some(new_handle()),
			)]))
			.id();

		app.world_mut()
			.run_system_once(visualize_slot_items::<_View>);

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&VisualizeCommands::<_View, SlotKey>::default().with_item(
				&SlotKey::BottomHand(Side::Right),
				None as Option<&SkillItem>
			)),
			entity.get::<VisualizeCommands<_View, SlotKey>>(),
		);
	}

	#[test]
	fn visualize_items() {
		let handle_a = new_handle();
		let handle_b = new_handle();
		let item_a = SkillItem {
			content: SkillItemContent {
				model: ModelRender::Hand(AssetModel::path("my hand model")),
				..default()
			},
			..default()
		};
		let item_b = SkillItem {
			content: SkillItemContent {
				model: ModelRender::Forearm(AssetModel::path("my forearm model")),
				..default()
			},
			..default()
		};
		let mut app = setup([
			(handle_a.id(), item_a.clone()),
			(handle_b.id(), item_b.clone()),
		]);
		let entity = app
			.world_mut()
			.spawn(Slots::new([
				(SlotKey::BottomHand(Side::Right), Some(handle_a)),
				(SlotKey::TopHand(Side::Right), Some(handle_b)),
			]))
			.id();

		app.world_mut()
			.run_system_once(visualize_slot_items::<_View>);

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(
				&VisualizeCommands::<_View, SlotKey>::default()
					.with_item(&SlotKey::BottomHand(Side::Right), Some(&item_a))
					.with_item(&SlotKey::TopHand(Side::Right), Some(&item_b))
			),
			entity.get::<VisualizeCommands<_View, SlotKey>>(),
		);
	}

	#[test]
	fn visualize_item_only_once() {
		let handle = new_handle();
		let item = SkillItem {
			content: SkillItemContent {
				model: ModelRender::Hand(AssetModel::path("my model")),
				..default()
			},
			..default()
		};
		let mut app = setup([(handle.id(), item.clone())]);
		let entity = app
			.world_mut()
			.spawn(Slots::new([(
				SlotKey::BottomHand(Side::Right),
				Some(handle.clone()),
			)]))
			.id();

		app.add_systems(Update, visualize_slot_items::<_View>);
		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<VisualizeCommands<_View, SlotKey>>();
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(None, entity.get::<VisualizeCommands<_View, SlotKey>>());
	}

	#[test]
	fn visualize_items_again_after_slots_mutably_dereferenced() {
		let handle = new_handle();
		let item = SkillItem {
			content: SkillItemContent {
				model: ModelRender::Hand(AssetModel::path("my other model")),
				..default()
			},
			..default()
		};
		let mut app = setup([(handle.id(), item.clone())]);
		let entity = app
			.world_mut()
			.spawn(Slots::new([(
				SlotKey::BottomHand(Side::Right),
				Some(handle.clone()),
			)]))
			.id();

		app.add_systems(Update, visualize_slot_items::<_View>);
		app.update();
		let mut agent = app.world_mut().entity_mut(entity);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		_ = slots.deref_mut();
		agent.remove::<VisualizeCommands<_View, SlotKey>>();
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(
				&VisualizeCommands::<_View, SlotKey>::default()
					.with_item(&SlotKey::BottomHand(Side::Right), Some(&item))
			),
			entity.get::<VisualizeCommands<_View, SlotKey>>(),
		);
	}
}
