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
	resources::Shared,
	traits::{
		load_asset::{LoadAsset, Path},
		try_insert_on::TryInsertOn,
	},
};
use skills::{
	components::slots::Slots,
	items::SlotKey,
	skills::{Queued, Skill},
	traits::{IsLingering, Iter, PeekNext},
};

type PlayerComponents<'a, TQueue, TCombos, TComboLinger> = (
	&'a Slots,
	&'a TQueue,
	Option<&'a TCombos>,
	Option<&'a TComboLinger>,
);

type IconPath = Option<fn() -> Path>;

pub fn quickbar<
	TQueue: Component + Iter<Skill<Queued>>,
	TCombos: Component + PeekNext<Skill>,
	TComboLinger: Component + IsLingering,
	TServer: Resource + LoadAsset<Image>,
>(
	mut commands: Commands,
	mut icons: ResMut<Shared<Path, Icon>>,
	server: ResMut<TServer>,
	players: Query<PlayerComponents<TQueue, TCombos, TComboLinger>, With<Player>>,
	mut panels: Query<(Entity, &mut QuickbarPanel)>,
) {
	let Ok((slots, queue, combos, combo_linger)) = players.get_single() else {
		return;
	};
	let icons = icons.as_mut();
	let server = server.as_ref();
	let mut get_icon_image = |key: &SlotKey| {
		icon_of_active_skill(key, queue)
			.or_else(icon_of_lingering_combo(key, slots, combos, combo_linger))
			.or_else(icon_of_slot_item(key, slots))
			.and_then(load_image(icons, server))
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

fn icon_of_active_skill<TQueue: Iter<Skill<Queued>>>(
	slot_key: &SlotKey,
	queue: &TQueue,
) -> Option<IconPath> {
	queue
		.iter()
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

fn load_image<'a, TServer: Resource + LoadAsset<Image>>(
	icons: &'a mut Shared<Path, Icon>,
	server: &'a TServer,
) -> impl FnOnce(IconPath) -> Option<Icon> + 'a {
	|icon| {
		let icon = icon?;
		Some(icons.get_handle(icon(), || Icon(server.load_asset(icon()))))
	}
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
		traits::{IsLingering, Iter, PeekNext},
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

	impl Iter<Skill<Queued>> for _Queue {
		fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a Skill<Queued>>
		where
			Skill<Queued>: 'a,
		{
			self.0.iter()
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
	struct _Server {
		mock: Mock_Server,
	}

	#[automock]
	impl LoadAsset<Image> for _Server {
		fn load_asset(&self, path: Path) -> Handle<Image> {
			self.mock.load_asset(path)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Shared<Path, Icon>>();
		app.add_systems(Update, quickbar::<_Queue, _Combos, _ComboLingers, _Server>);

		app
	}

	fn mounts() -> Mounts<Entity> {
		Mounts {
			hand: Entity::from_raw(100),
			forearm: Entity::from_raw(200),
		}
	}

	#[test]
	fn add_item_skill_icon() {
		let mut app = setup();
		let mut server = _Server::default();
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

		server.mock.expect_load_asset().return_const(handle.clone());
		app.insert_resource(server);

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
	fn store_item_skill_icon_in_shared() {
		let mut app = setup();
		let mut server = _Server::default();
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

		server.mock.expect_load_asset().return_const(handle.clone());
		app.insert_resource(server);

		app.world.spawn((Player, slots, _Queue::default()));
		app.world.spawn(QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Empty,
		});

		app.update();

		let shared = app.world.resource::<Shared<Path, Icon>>();

		assert_eq!(
			Some(&Icon(handle)),
			shared.get(&Path::from("item_skill/icon/path"))
		);
	}

	#[test]
	fn load_item_skill_icon_from_shared() {
		let mut app = setup();
		let mut server = _Server::default();
		let mut shared = Shared::default();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("skill_item/icon/path"))),
			},
		)]));

		server
			.mock
			.expect_load_asset()
			.never()
			.return_const(Handle::default());
		app.insert_resource(server);

		shared.get_handle(Path::from("skill_item/icon/path"), || Icon(handle.clone()));
		app.insert_resource(shared);

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
	fn load_with_item_skill_icon_path() {
		let mut app = setup();
		let mut server = _Server::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

		server
			.mock
			.expect_load_asset()
			.times(1)
			.with(eq(Path::from("item_skill/icon/path")))
			.return_const(Handle::default());
		app.insert_resource(server);

		app.world.spawn((Player, slots, _Queue::default()));
		app.world.spawn(QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Empty,
		});

		app.update();
	}

	#[test]
	fn add_queued_skill_icon() {
		let mut app = setup();
		let mut server = _Server::default();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("icon/path"))),
			},
		)]));

		server.mock.expect_load_asset().return_const(handle.clone());
		app.insert_resource(server);

		app.world.spawn((
			Player,
			slots,
			_Queue(vec![Skill::with_icon_path(|| Path::from("icon/path"))
				.with(Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				})]),
		));
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
	fn load_with_queued_skill_icon_path() {
		let mut app = setup();
		let mut server = _Server::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));
		let queue = _Queue(vec![Skill::with_icon_path(|| {
			Path::from("queued_skill/icon/path")
		})
		.with(Queued {
			slot_key: SlotKey::Hand(Side::Main),
			..default()
		})]);

		server
			.mock
			.expect_load_asset()
			.times(1)
			.with(eq(Path::from("queued_skill/icon/path")))
			.return_const(Handle::default());
		app.insert_resource(server);

		app.world.spawn((Player, slots, queue));
		app.world.spawn(QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Empty,
		});

		app.update();
	}

	#[test]
	fn store_queued_skill_icon_in_shared() {
		let mut app = setup();
		let mut server = _Server::default();
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
		let queue = _Queue(vec![Skill::with_icon_path(|| {
			Path::from("queued_skill/icon/path")
		})
		.with(Queued {
			slot_key: SlotKey::Hand(Side::Main),
			..default()
		})]);

		server.mock.expect_load_asset().return_const(handle.clone());
		app.insert_resource(server);

		app.world.spawn((Player, slots, queue));
		app.world.spawn(QuickbarPanel {
			key: SlotKey::Hand(Side::Main),
			state: PanelState::Empty,
		});

		app.update();

		let shared = app.world.resource::<Shared<Path, Icon>>();

		assert_eq!(
			Some(&Icon(handle)),
			shared.get(&Path::from("queued_skill/icon/path"))
		);
	}

	#[test]
	fn load_queued_skill_icon_from_shared() {
		let mut app = setup();
		let mut server = _Server::default();
		let mut shared = Shared::default();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("skill_item/icon/path"))),
			},
		)]));
		let queue = _Queue(vec![Skill::with_icon_path(|| {
			Path::from("queued_skill/icon/path")
		})
		.with(Queued {
			slot_key: SlotKey::Hand(Side::Main),
			..default()
		})]);

		server
			.mock
			.expect_load_asset()
			.never()
			.return_const(Handle::default());
		app.insert_resource(server);

		shared.get_handle(Path::from("queued_skill/icon/path"), || {
			Icon(handle.clone())
		});
		app.insert_resource(shared);

		app.world.spawn((Player, slots, queue));
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
		let mut server = _Server::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));

		server
			.mock
			.expect_load_asset()
			.never()
			.return_const(Handle::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}));
		app.insert_resource(server);

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
		let mut server = _Server::default();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

		server
			.mock
			.expect_load_asset()
			.with(eq(Path::from("combo_skill/icon/path")))
			.return_const(Handle::default());
		app.insert_resource(server);

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
		let mut server = _Server::default();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

		server
			.mock
			.expect_load_asset()
			.with(eq(Path::from("item_skill/icon/path")))
			.return_const(Handle::default());
		app.insert_resource(server);

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
		let mut server = _Server::default();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

		server
			.mock
			.expect_load_asset()
			.with(eq(Path::from("active_skill/icon/path")))
			.return_const(Handle::default());
		app.insert_resource(server);

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
		let mut server = _Server::default();
		let mut combos = _Combos::default();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: mounts(),
				item: Some(Item::with_icon_path(|| Path::from("item_skill/icon/path"))),
			},
		)]));

		server
			.mock
			.expect_load_asset()
			.return_const(Handle::default());
		app.insert_resource(server);

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
