use crate::{components::QuickbarPanel, tools::PanelState};
use bevy::{
	asset::Handle,
	ecs::{
		component::Component,
		query::With,
		system::{Commands, Query, Res},
	},
	prelude::Entity,
	render::texture::Image,
	ui::UiImage,
};
use common::{components::Player, traits::try_insert_on::TryInsertOn};
use skills::{
	components::{SlotKey, Slots},
	resources::SkillIcons,
	skill::{Queued, Skill},
	traits::Iter,
};

pub fn quickbar<TQueue: Iter<Skill<Queued>> + Component>(
	mut commands: Commands,
	icons: Res<SkillIcons>,
	players: Query<(&Slots, &TQueue), With<Player>>,
	mut panels: Query<(Entity, &mut QuickbarPanel)>,
) {
	let default_icon = &Handle::default();
	let Ok((slots, queue)) = players.get_single() else {
		return;
	};

	for (id, mut panel) in &mut panels {
		let (state, image) = get_state_and_image(&panel.key, slots, queue, &icons, default_icon);

		panel.state = state;
		commands.try_insert_on(id, image);
	}
}

fn get_state_and_image<TQueue: Iter<Skill<Queued>>>(
	slot_key: &SlotKey,
	slots: &Slots,
	queue: &TQueue,
	icons: &Res<SkillIcons>,
	default_icon: &Handle<Image>,
) -> (PanelState, UiImage) {
	match get_icon(slots, slot_key, queue, icons, default_icon) {
		Some(skill_icon) => (PanelState::Filled, UiImage::new(skill_icon.clone())),
		None => (PanelState::Empty, UiImage::new(default_icon.clone())),
	}
}

fn get_icon<'a, TQueue: Iter<Skill<Queued>>>(
	slots: &Slots,
	slot_key: &SlotKey,
	queue: &TQueue,
	icons: &'a Res<SkillIcons>,
	default: &'a Handle<Image>,
) -> Option<&'a Handle<Image>> {
	let slot = slots.0.get(slot_key)?;
	let key = match &queue.iter().find(|s| &s.data.slot_key == slot_key) {
		Some(skill) => skill.name,
		None => {
			slot.item
				.as_ref()
				.and_then(|item| item.skill.as_ref())?
				.name
		}
	};

	Some(icons.0.get(&key).unwrap_or(default))
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
	use common::components::Side;
	use skills::{
		components::{Item, Slot},
		skill::{Queued, Skill},
		traits::Iter,
	};
	use std::collections::HashMap;

	trait WithSkillName {
		fn with_skill_name(skill_name: &'static str) -> Self;
	}

	impl WithSkillName for Item {
		fn with_skill_name(skill_name: &'static str) -> Self {
			Item {
				skill: Some(Skill::with_skill_name(skill_name)),
				..default()
			}
		}
	}

	impl WithSkillName for Skill {
		fn with_skill_name(skill_name: &'static str) -> Self {
			Skill {
				name: skill_name,
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

	#[test]
	fn add_item_skill_icon() {
		let mut app = App::new();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let icons = SkillIcons(HashMap::from([("my skill", handle.clone())]));
		app.insert_resource(icons);
		app.world.spawn((
			Player,
			Slots(HashMap::from([(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item::with_skill_name("my skill")),
				},
			)])),
			_Queue::default(),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.add_systems(Update, quickbar::<_Queue>);
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
	fn add_queued_skill_icon() {
		let mut app = App::new();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let icons = SkillIcons(HashMap::from([
			("my skill", Handle::default()),
			("my combo skill", handle.clone()),
		]));
		app.insert_resource(icons);
		app.world.spawn((
			Player,
			Slots(HashMap::from([(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item::with_skill_name("my skill")),
				},
			)])),
			_Queue(vec![Skill::with_skill_name("my combo skill").with(
				Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
			)]),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.add_systems(Update, quickbar::<_Queue>);
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
	fn ignore_queued_skill_icon_when_key_not_matching() {
		let mut app = App::new();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let icons = SkillIcons(HashMap::from([
			("my skill", handle.clone()),
			("my combo skill", Handle::default()),
		]));
		app.insert_resource(icons);
		app.world.spawn((
			Player,
			Slots(HashMap::from([(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item::with_skill_name("my skill")),
				},
			)])),
			_Queue(vec![Skill::with_skill_name("my combo skill").with(
				Queued {
					slot_key: SlotKey::Hand(Side::Off),
					..default()
				},
			)]),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.add_systems(Update, quickbar::<_Queue>);
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
	fn add_default_image_when_icon_not_found() {
		let mut app = App::new();
		let icons = SkillIcons(HashMap::from([]));
		app.insert_resource(icons);
		app.world.spawn((
			Player,
			Slots(HashMap::from([(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item::with_skill_name("my skill")),
				},
			)])),
			_Queue::default(),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.add_systems(Update, quickbar::<_Queue>);
		app.update();

		let panel = app.world.entity(panel);

		assert_eq!(
			(Some(Handle::default()), Some(PanelState::Filled)),
			(
				panel.get::<UiImage>().map(|image| image.texture.clone()),
				panel.get::<QuickbarPanel>().map(|panel| panel.state)
			)
		);
	}

	#[test]
	fn set_empty_when_no_skill_found() {
		let mut app = App::new();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let icons = SkillIcons(HashMap::from([("my skill", handle.clone())]));
		app.insert_resource(icons);
		app.world.spawn((
			Player,
			Slots(HashMap::from([(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: None,
				},
			)])),
			_Queue::default(),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Filled,
			})
			.id();

		app.add_systems(Update, quickbar::<_Queue>);
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
