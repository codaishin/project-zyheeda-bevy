use crate::{
	components::quickbar_panel::QuickbarPanel,
	tools::{Icon, PanelState},
};
use bevy::{
	ecs::{
		component::Component,
		query::With,
		system::{Commands, Query, ResMut, Resource},
	},
	prelude::Entity,
	render::texture::Image,
	ui::UiImage,
	utils::default,
};
use common::{
	components::Player,
	traits::{
		iterate::Iterate,
		load_asset::Path,
		shared_asset_handle::SharedAssetHandle,
		try_insert_on::TryInsertOn,
	},
};
use skills::{
	components::slots::Slots,
	items::slot_key::SlotKey,
	skills::{Queued, Skill},
	traits::{IsLingering, PeekNext},
};

type PlayerComponents<'a, TQueue, TCombos, TComboLinger> = (
	&'a Slots,
	&'a TQueue,
	Option<&'a TCombos>,
	Option<&'a TComboLinger>,
);

type IconPath = Option<fn() -> Path>;

pub fn quickbar<
	TQueue: Component + Iterate<Skill<Queued>>,
	TCombos: Component + PeekNext<Skill>,
	TComboLinger: Component + IsLingering,
	TAssets: Resource + SharedAssetHandle<TCache, Path, Image>,
	TCache: Resource,
>(
	mut commands: Commands,
	mut assets: ResMut<TAssets>,
	mut cache: ResMut<TCache>,
	players: Query<PlayerComponents<TQueue, TCombos, TComboLinger>, With<Player>>,
	mut panels: Query<(Entity, &mut QuickbarPanel)>,
) {
	let Ok((slots, queue, combos, combo_linger)) = players.get_single() else {
		return;
	};
	let assets = assets.as_mut();
	let cache = cache.as_mut();
	let mut get_icon_image = |key: &SlotKey| {
		icon_of_active_skill(key, queue)
			.or_else(icon_of_lingering_combo(key, slots, combos, combo_linger))
			.or_else(icon_of_slot_item(key, slots))
			.and_then(load_image(assets, cache))
	};

	for (id, mut panel) in &mut panels {
		let (state, image) = match get_icon_image(&panel.key) {
			Some(image) => (PanelState::Filled, UiImage::new(image.0)),
			None => (PanelState::Empty, UiImage::new(default())),
		};

		panel.state = state;
		commands.try_insert_on(id, image);
	}
}

fn icon_of_active_skill<TQueue: Iterate<Skill<Queued>>>(
	slot_key: &SlotKey,
	queue: &TQueue,
) -> Option<IconPath> {
	queue
		.iterate()
		.find(|s| &s.data.slot_key == slot_key)
		.map(|s| s.icon)
}

fn icon_of_lingering_combo<'a, TCombos: PeekNext<Skill>, TComboLinger: IsLingering>(
	slot_key: &'a SlotKey,
	slots: &'a Slots,
	combos: Option<&'a TCombos>,
	combo_linger: Option<&'a TComboLinger>,
) -> impl FnOnce() -> Option<IconPath> + 'a {
	move || {
		if !combo_linger?.is_lingering() {
			return None;
		}

		combos?.peek_next(slot_key, slots).map(|s| s.icon)
	}
}

fn icon_of_slot_item<'a>(
	slot_key: &'a SlotKey,
	slots: &'a Slots,
) -> impl FnOnce() -> Option<IconPath> + 'a {
	|| {
		slots
			.0
			.get(slot_key)?
			.item
			.as_ref()
			.map(|item| item.skill.as_ref()?.icon)
	}
}

fn load_image<'a, TAssets: Resource + SharedAssetHandle<TCache, Path, Image>, TCache>(
	assets: &'a mut TAssets,
	cache: &'a mut TCache,
) -> impl FnOnce(IconPath) -> Option<Icon> + 'a {
	|icon| Some(Icon(assets.handle(cache, icon?())))
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		ecs::{component::Component, entity::Entity},
		ui::UiImage,
		utils::{default, Uuid},
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use mockall::{automock, predicate::eq};
	use skills::{
		components::{Mounts, Slot},
		items::Item,
		skills::{Queued, Skill},
		traits::{IsLingering, PeekNext},
	};
	use std::collections::HashMap;

	trait _WithIconPath {
		fn with_icon_path(path: fn() -> Path) -> Self;
	}

	impl _WithIconPath for Item {
		fn with_icon_path(path: fn() -> Path) -> Self {
			Item {
				skill: Some(Skill::with_icon_path(path)),
				..default()
			}
		}
	}

	impl _WithIconPath for Skill {
		fn with_icon_path(path: fn() -> Path) -> Self {
			Skill {
				icon: Some(path),
				..default()
			}
		}
	}

	impl _WithIconPath for Skill<Queued> {
		fn with_icon_path(path: fn() -> Path) -> Self {
			Skill {
				icon: Some(path),
				..default()
			}
		}
	}

	#[derive(Component, Default)]
	struct _Queue(Vec<Skill<Queued>>);

	impl Iterate<Skill<Queued>> for _Queue {
		fn iterate<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a Skill<Queued>>
		where
			Skill<Queued>: 'a,
		{
			self.0.iterate()
		}
	}

	#[derive(Component, Default)]
	struct _Combos {
		mock: Mock_Combos,
	}

	#[automock]
	impl PeekNext<Skill> for _Combos {
		fn peek_next(&self, trigger: &SlotKey, slots: &Slots) -> Option<Skill> {
			self.mock.peek_next(trigger, slots)
		}
	}

	#[derive(Component)]
	struct _ComboLingers(bool);

	impl IsLingering for _ComboLingers {
		fn is_lingering(&self) -> bool {
			self.0
		}
	}

	#[derive(Resource, Default, Debug, PartialEq, Clone, Copy)]
	struct _Cache(u32);

	#[derive(Resource, Default)]
	struct _Assets {
		mock: Mock_Assets,
	}

	#[automock]
	impl SharedAssetHandle<_Cache, Path, Image> for _Assets {
		fn handle(&mut self, cache: &mut _Cache, key: Path) -> Handle<Image> {
			self.mock.handle(cache, key)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Cache>();
		app.add_systems(
			Update,
			quickbar::<_Queue, _Combos, _ComboLingers, _Assets, _Cache>,
		);

		app
	}

	fn mounts() -> Mounts<Entity> {
		Mounts {
			hand: Entity::from_raw(100),
			forearm: Entity::from_raw(200),
		}
	}

	#[test]
	fn add_icon_image() {
		let mut app = setup();
		let mut assets = _Assets::default();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

		assets.mock.expect_handle().return_const(handle.clone());
		app.insert_resource(assets);

		app.world.spawn((Player, slots, _Queue::default()));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let panel = app.world.entity(panel);

		assert_eq!(
			(Some(handle), Some(PanelState::Filled)),
			(
				panel.get::<UiImage>().map(|image| image.texture.clone()),
				panel.get::<QuickbarPanel>().map(|panel| panel.state)
			)
		);
	}

	#[test]
	fn set_empty_when_no_skill_found() {
		let mut app = setup();
		let mut assets = _Assets::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));

		assets
			.mock
			.expect_handle()
			.never()
			.return_const(Handle::default());
		app.insert_resource(assets);

		app.world.spawn((Player, slots, _Queue::default()));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Filled,
			})
			.id();

		app.update();

		let panel = app.world.entity(panel);

		assert_eq!(
			(Some(Handle::default()), Some(PanelState::Empty)),
			(
				panel.get::<UiImage>().map(|image| image.texture.clone()),
				panel.get::<QuickbarPanel>().map(|panel| panel.state)
			)
		);
	}

	#[test]
	fn add_combo_skill_icon_when_no_skill_active_and_lingering() {
		let mut app = setup();
		let mut assets = _Assets::default();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));
		let cache = _Cache(42);

		assets
			.mock
			.expect_handle()
			.with(eq(cache), eq(Path::from("combo_skill/icon/path")))
			.return_const(Handle::default());
		app.insert_resource(cache);
		app.insert_resource(assets);

		combos
			.mock
			.expect_peek_next()
			.return_const(Skill::with_icon_path(|| {
				Path::from("combo_skill/icon/path")
			}));
		app.world.spawn((
			Player,
			slots,
			_Queue::default(),
			combos,
			_ComboLingers(true),
		));
		app.world.spawn(QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Empty,
		});

		app.update();
	}

	#[test]
	fn do_not_add_combo_skill_icon_when_no_skill_active_and_not_lingering() {
		let mut app = setup();
		let mut assets = _Assets::default();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));
		let cache = _Cache(42);

		assets
			.mock
			.expect_handle()
			.with(eq(cache), eq(Path::from("item_skill/icon/path")))
			.return_const(Handle::default());
		app.insert_resource(cache);
		app.insert_resource(assets);

		combos
			.mock
			.expect_peek_next()
			.return_const(Skill::with_icon_path(|| {
				Path::from("combo_skill/icon/path")
			}));
		app.world.spawn((
			Player,
			slots,
			_Queue::default(),
			combos,
			_ComboLingers(false),
		));
		app.world.spawn(QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Empty,
		});

		app.update();
	}

	#[test]
	fn do_not_add_combo_skill_icon_when_skill_active() {
		let mut app = setup();
		let mut assets = _Assets::default();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));
		let cache = _Cache(42);

		assets
			.mock
			.expect_handle()
			.with(eq(cache), eq(Path::from("active_skill/icon/path")))
			.return_const(Handle::default());
		app.insert_resource(cache);
		app.insert_resource(assets);

		combos
			.mock
			.expect_peek_next()
			.return_const(Skill::with_icon_path(|| {
				Path::from("combo_skill/icon/path")
			}));
		app.world.spawn((
			Player,
			slots,
			_Queue(vec![Skill::with_icon_path(|| {
				Path::from("active_skill/icon/path")
			})]),
			combos,
			_ComboLingers(true),
		));
		app.world.spawn(QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Empty,
		});

		app.update();
	}

	#[test]
	fn call_combo_peek_next_with_correct_args() {
		let mut app = setup();
		let mut assets = _Assets::default();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

		assets.mock.expect_handle().return_const(Handle::default());
		app.insert_resource(assets);

		combos
			.mock
			.expect_peek_next()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Off)), eq(slots.clone()))
			.return_const(Skill::with_icon_path(|| {
				Path::from("combo_skill/icon/path")
			}));
		app.world.spawn((
			Player,
			slots,
			_Queue::default(),
			combos,
			_ComboLingers(true),
		));
		app.world.spawn(QuickbarPanel {
			key: SlotKey::Hand(Side::Off),
			state: PanelState::Empty,
		});

		app.update();
	}
}
