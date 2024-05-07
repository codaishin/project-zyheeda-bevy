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
	traits::Iter,
};

pub fn quickbar<TQueue: Iter<Skill<Queued>> + Component, TServer: Resource + LoadAsset<Image>>(
	mut commands: Commands,
	mut icons: ResMut<Shared<Path, Icon>>,
	mut server: ResMut<TServer>,
	players: Query<(&Slots, &TQueue), With<Player>>,
	mut panels: Query<(Entity, &mut QuickbarPanel)>,
) {
	let Ok((slots, queue)) = players.get_single() else {
		return;
	};
	let icons = icons.as_mut();
	let server = server.as_mut();

	for (id, mut panel) in &mut panels {
		let (state, image) = get_state_and_image(&panel.key, slots, queue, icons, server);

		panel.state = state;
		commands.try_insert_on(id, image);
	}
}

fn get_state_and_image<TQueue: Iter<Skill<Queued>>, TServer: LoadAsset<Image>>(
	slot_key: &SlotKey,
	slots: &Slots,
	queue: &TQueue,
	icons: &mut Shared<Path, Icon>,
	server: &mut TServer,
) -> (PanelState, UiImage) {
	match get_icon(slots, slot_key, queue, icons, server) {
		Some(icon) => (PanelState::Filled, UiImage::new(icon.0)),
		None => (PanelState::Empty, UiImage::new(default())),
	}
}

fn get_icon<'a, TQueue: Iter<Skill<Queued>>, TServer: LoadAsset<Image>>(
	slots: &Slots,
	slot_key: &SlotKey,
	queue: &TQueue,
	icons: &'a mut Shared<Path, Icon>,
	server: &'a mut TServer,
) -> Option<Icon> {
	let slot = slots.0.get(slot_key)?;
	let path = match &queue.iter().find(|s| &s.data.slot_key == slot_key) {
		Some(skill) => skill.icon,
		None => {
			slot.item
				.as_ref()
				.and_then(|item| item.skill.as_ref())?
				.icon
		}
	};
	let path = path?;
	let load_icon = || Icon(server.load_asset(path()));

	Some(icons.get_handle(path(), load_icon))
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
		traits::Iter,
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
		app.add_systems(Update, quickbar::<_Queue, _Server>);

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
}
