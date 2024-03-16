use crate::{components::QuickbarPanel, tools::PanelState};
use bevy::{
	asset::Handle,
	ecs::{
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
};

pub fn quickbar(
	mut commands: Commands,
	icons: Res<SkillIcons>,
	players: Query<&Slots, With<Player>>,
	mut panels: Query<(Entity, &mut QuickbarPanel)>,
) {
	let default_icon = &Handle::default();
	let Ok(slots) = players.get_single() else {
		return;
	};

	for (id, mut panel) in &mut panels {
		let (state, image) = get_state_and_image(&panel.key, slots, &icons, default_icon);

		panel.state = state;
		commands.try_insert_on(id, image);
	}
}

fn get_state_and_image(
	slot_key: &SlotKey,
	slots: &Slots,
	icons: &Res<SkillIcons>,
	default_icon: &Handle<Image>,
) -> (PanelState, UiImage) {
	match get_icon(slots, slot_key, icons, default_icon) {
		Some(skill_icon) => (PanelState::Filled, UiImage::new(skill_icon.clone())),
		None => (PanelState::Empty, UiImage::new(default_icon.clone())),
	}
}

fn get_icon<'a>(
	slots: &Slots,
	slot_key: &SlotKey,
	icons: &'a Res<SkillIcons>,
	default: &'a Handle<Image>,
) -> Option<&'a Handle<Image>> {
	let slot = slots.0.get(slot_key)?;
	let key = match &slot.combo_skill {
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
		ecs::entity::Entity,
		ui::UiImage,
		utils::{default, Uuid},
	};
	use common::components::Side;
	use skills::{
		components::{Item, Slot},
		skill::Skill,
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
					combo_skill: None,
				},
			)])),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.add_systems(Update, quickbar);
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
	fn add_slot_combo_skill_icon() {
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
					combo_skill: Some(Skill::with_skill_name("my combo skill")),
				},
			)])),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.add_systems(Update, quickbar);
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
					combo_skill: Some(Skill::with_skill_name("my combo skill")),
				},
			)])),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.add_systems(Update, quickbar);
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
					combo_skill: None,
				},
			)])),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Filled,
			})
			.id();

		app.add_systems(Update, quickbar);
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
