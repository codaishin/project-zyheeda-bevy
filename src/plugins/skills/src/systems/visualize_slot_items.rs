use crate::{components::slots::Slots, item::SkillItemContent, slot_key::SlotKey};
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
) where
	TView: ItemView<SlotKey> + Sync + Send + 'static,
	SkillItemContent: GetViewData<TView, SlotKey>,
{
	for (entity, slots) in &agents {
		let mut visualize = VisualizeCommands::<TView, SlotKey>::default();

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
		components::renderer::{ModelRender, Renderer},
		item::{SkillItem, SkillItemContent},
		slot_key::SlotKey,
	};
	use bevy::ecs::system::RunSystemOnce;
	use common::components::{AssetModel, Side};
	use items::components::visualize::VisualizeCommands;

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

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn visualize_item() {
		let mut app = setup();
		let item = SkillItem {
			content: SkillItemContent {
				render: Renderer {
					model: ModelRender::Hand(AssetModel::Path("my model")),
					..default()
				},
				..default()
			},
			..default()
		};
		let entity = app
			.world_mut()
			.spawn(Slots::new([(
				SlotKey::BottomHand(Side::Right),
				Some(item.clone()),
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
	fn visualize_items() {
		let mut app = setup();
		let item_a = SkillItem {
			content: SkillItemContent {
				render: Renderer {
					model: ModelRender::Hand(AssetModel::Path("my hand model")),
					..default()
				},
				..default()
			},
			..default()
		};
		let item_b = SkillItem {
			content: SkillItemContent {
				render: Renderer {
					model: ModelRender::Forearm(AssetModel::Path("my forearm model")),
					..default()
				},
				..default()
			},
			..default()
		};
		let entity = app
			.world_mut()
			.spawn(Slots::new([
				(SlotKey::BottomHand(Side::Right), Some(item_a.clone())),
				(SlotKey::TopHand(Side::Right), Some(item_b.clone())),
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
		let mut app = setup();
		let item = SkillItem {
			content: SkillItemContent {
				render: Renderer {
					model: ModelRender::Hand(AssetModel::Path("my model")),
					..default()
				},
				..default()
			},
			..default()
		};
		let entity = app
			.world_mut()
			.spawn(Slots::new([(
				SlotKey::BottomHand(Side::Right),
				Some(item.clone()),
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
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Slots::new([(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem {
					content: SkillItemContent {
						render: Renderer {
							model: ModelRender::Hand(AssetModel::Path("my model")),
							..default()
						},
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
				render: Renderer {
					model: ModelRender::Hand(AssetModel::Path("my other model")),
					..default()
				},
				..default()
			},
			..default()
		};
		*slots = Slots::new([(SlotKey::TopHand(Side::Right), Some(item.clone()))]);
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(
				&VisualizeCommands::<_View, SlotKey>::default()
					.with_item(&SlotKey::TopHand(Side::Right), Some(&item))
			),
			entity.get::<VisualizeCommands<_View, SlotKey>>(),
		);
	}
}
