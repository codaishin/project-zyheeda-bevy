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
		cache::{GetOrLoadAsset, GetOrLoadAssetFactory},
		iterate::Iterate,
		load_asset::Path,
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

pub fn quickbar<TQueue, TCombos, TComboLinger, TAssets, TStorage, TFactory>(
	mut commands: Commands,
	assets: ResMut<TAssets>,
	storage: ResMut<TStorage>,
	players: Query<PlayerComponents<TQueue, TCombos, TComboLinger>, With<Player>>,
	mut panels: Query<(Entity, &mut QuickbarPanel)>,
) where
	TQueue: Component + Iterate<Skill<Queued>>,
	TCombos: Component + PeekNext<Skill>,
	TComboLinger: Component + IsLingering,
	TAssets: Resource,
	TStorage: Resource,
	TFactory: GetOrLoadAssetFactory<TAssets, Image, TStorage>,
{
	let Ok((slots, queue, combos, combo_linger)) = players.get_single() else {
		return;
	};
	let cache = &mut TFactory::create_from(assets, storage);
	let mut get_icon_image = |key: &SlotKey| {
		icon_of_active_skill(key, queue)
			.or_else(icon_of_lingering_combo(key, slots, combos, combo_linger))
			.or_else(icon_of_slot_item(key, slots))
			.and_then(load_image(cache))
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

fn load_image(
	cache: &mut impl GetOrLoadAsset<Image>,
) -> impl FnOnce(IconPath) -> Option<Icon> + '_ {
	|icon| Some(Icon(cache.get_or_load(icon?())))
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
	use mockall::{automock, mock, predicate::eq};
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

	#[derive(Resource, Default)]
	struct _Storage;

	#[derive(Resource, Default)]
	struct _Assets;

	mock! {
		_Cache {}
		impl GetOrLoadAsset<Image> for _Cache {
			fn get_or_load(&mut self, key: Path) -> Handle<Image>;
		}
	}

	fn setup<TFactory>() -> App
	where
		for<'a> TFactory: GetOrLoadAssetFactory<_Assets, Image, _Storage> + 'static,
	{
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Assets>();
		app.init_resource::<_Storage>();
		app.add_systems(
			Update,
			quickbar::<_Queue, _Combos, _ComboLingers, _Assets, _Storage, TFactory>,
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
		const HANDLE: Handle<Image> = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::from_u128(0xe5db01e4_9a32_43d1_b048_d690d646adde),
		});

		struct _Factory;

		impl GetOrLoadAssetFactory<_Assets, Image, _Storage> for _Factory {
			fn create_from(_: ResMut<_Assets>, _: ResMut<_Storage>) -> impl GetOrLoadAsset<Image> {
				let mut cache = Mock_Cache::default();
				cache.expect_get_or_load().return_const(HANDLE);
				cache
			}
		}

		let mut app = setup::<_Factory>();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

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
			(Some(HANDLE), Some(PanelState::Filled)),
			(
				panel.get::<UiImage>().map(|image| image.texture.clone()),
				panel.get::<QuickbarPanel>().map(|panel| panel.state)
			)
		);
	}

	#[test]
	fn set_empty_when_no_skill_found() {
		struct _Factory;

		impl GetOrLoadAssetFactory<_Assets, Image, _Storage> for _Factory {
			fn create_from(_: ResMut<_Assets>, _: ResMut<_Storage>) -> impl GetOrLoadAsset<Image> {
				let mut cache = Mock_Cache::default();
				cache.expect_get_or_load().return_const(Handle::default());
				cache
			}
		}

		let mut app = setup::<_Factory>();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));

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
		struct _Factory;

		impl GetOrLoadAssetFactory<_Assets, Image, _Storage> for _Factory {
			fn create_from(_: ResMut<_Assets>, _: ResMut<_Storage>) -> impl GetOrLoadAsset<Image> {
				let mut cache = Mock_Cache::default();
				cache
					.expect_get_or_load()
					.with(eq(Path::from("combo_skill/icon/path")))
					.return_const(Handle::default());
				cache
			}
		}

		let mut combos = _Combos::default();
		let mut app = setup::<_Factory>();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

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
	fn add_item_skill_icon_when_no_skill_active_and_not_lingering() {
		struct _Factory;

		impl GetOrLoadAssetFactory<_Assets, Image, _Storage> for _Factory {
			fn create_from(_: ResMut<_Assets>, _: ResMut<_Storage>) -> impl GetOrLoadAsset<Image> {
				let mut cache = Mock_Cache::default();
				cache
					.expect_get_or_load()
					.with(eq(Path::from("item_skill/icon/path")))
					.return_const(Handle::default());
				cache
			}
		}

		let mut app = setup::<_Factory>();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

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
	fn add_active_skill_icon_when_skill_active() {
		struct _Factory;

		impl GetOrLoadAssetFactory<_Assets, Image, _Storage> for _Factory {
			fn create_from(_: ResMut<_Assets>, _: ResMut<_Storage>) -> impl GetOrLoadAsset<Image> {
				let mut cache = Mock_Cache::default();
				cache
					.expect_get_or_load()
					.with(eq(Path::from("active_skill/icon/path")))
					.return_const(Handle::default());
				cache
			}
		}

		let mut app = setup::<_Factory>();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

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
		struct _Factory;

		impl GetOrLoadAssetFactory<_Assets, Image, _Storage> for _Factory {
			fn create_from(_: ResMut<_Assets>, _: ResMut<_Storage>) -> impl GetOrLoadAsset<Image> {
				let mut cache = Mock_Cache::default();
				cache.expect_get_or_load().return_const(Handle::default());
				cache
			}
		}

		let mut app = setup::<_Factory>();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

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
