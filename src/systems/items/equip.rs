use crate::{
	components::{Equip, Item, SlotKey, Slots},
	resources::Models,
};
use bevy::{
	prelude::{error, info, Commands, Entity, Handle, Query, Res},
	scene::Scene,
};
use std::borrow::Cow;

enum NoMatching {
	Slot(SlotKey),
	SceneHandle(Entity),
	Model(Cow<'static, str>),
}

type ShouldRetry = bool;

fn try_set_model(
	handles: Result<(bevy::prelude::Mut<Handle<Scene>>, &Handle<Scene>), NoMatching>,
	item: &Item,
) -> ShouldRetry {
	match handles {
		Ok((mut slot_handle, model_handle)) => {
			*slot_handle = model_handle.clone();
			false
		}
		Err(NoMatching::Model(model_key)) => {
			error!("{:?}: no model found for {:?}, abandoning", item, model_key);
			false
		}
		Err(NoMatching::Slot(slot)) => {
			info!(
				"{:?}: slot {:?} not found, retrying next update",
				item, slot,
			);
			true
		}
		Err(NoMatching::SceneHandle(slot)) => {
			error!("{:?}: {:?} has no Handle<Scene>, abandoning", item, slot);
			false
		}
	}
}

fn equip_item_to(
	slots: &Slots,
	item: &Item,
	models: &Res<Models>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
) -> ShouldRetry {
	let handles = slots
		.0
		.get(&item.slot)
		.ok_or(NoMatching::Slot(item.slot))
		.and_then(|slot| match scene_handles.get_mut(*slot) {
			Ok(slot_handle) => Ok(slot_handle),
			Err(_) => Err(NoMatching::SceneHandle(*slot)),
		})
		.and_then(|slot_handle| match models.0.get(&item.model) {
			Some(model_handle) => Ok((slot_handle, model_handle)),
			None => Err(NoMatching::Model(item.model.clone())),
		});

	try_set_model(handles, item)
}

fn equip_items_to(
	slots: &Slots,
	equip: &Equip,
	models: &Res<'_, Models>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
) -> Vec<Item> {
	equip
		.0
		.iter()
		.filter(|item| equip_item_to(slots, item, models, scene_handles))
		.cloned()
		.collect()
}

pub fn equip_items(
	mut commands: Commands,
	models: Res<Models>,
	mut agent: Query<(Entity, &Slots, &mut Equip)>,
	mut scene_handles: Query<&mut Handle<Scene>>,
) {
	for (agent, slots, mut equip) in &mut agent {
		let remainder = equip_items_to(slots, &equip, &models, &mut scene_handles);
		if remainder.is_empty() {
			commands.entity(agent).remove::<Equip>();
		} else {
			equip.0 = remainder;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Item, Side, SlotKey, Slots},
		resources::Models,
	};
	use bevy::{
		asset::HandleId,
		prelude::{App, Handle, Update},
		scene::Scene,
		utils::Uuid,
	};
	use std::borrow::Cow;

	#[test]
	fn equip_when_marked_to_equip() {
		let model_handle = Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 42));
		let models = Models([(Cow::from("model key"), model_handle.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app
			.world
			.spawn(Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 11)))
			.id();
		app.world.spawn((
			Slots([(SlotKey::Hand(Side::Right), slot)].into()),
			Equip::new([Item {
				slot: SlotKey::Hand(Side::Right),
				model: "model key".into(),
			}]),
		));
		app.add_systems(Update, equip_items);

		app.update();

		let slot_handle = app.world.entity(slot).get::<Handle<Scene>>();

		assert_eq!(Some(model_handle), slot_handle.cloned());
	}

	#[test]
	fn remove_equip_component() {
		let model_handle = Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 42));
		let models = Models([(Cow::from("model key"), model_handle.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app
			.world
			.spawn(Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 11)))
			.id();
		let agent = app
			.world
			.spawn((
				Slots([(SlotKey::Hand(Side::Right), slot)].into()),
				Equip::new([Item {
					slot: SlotKey::Hand(Side::Right),
					model: "model key".into(),
				}]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Equip>());
	}

	#[test]
	fn remove_equip_component_when_no_slot_scene_handle() {
		let model_handle = Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 42));
		let models = Models([(Cow::from("model key"), model_handle.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app.world.spawn(()).id();
		let agent = app
			.world
			.spawn((
				Slots([(SlotKey::Hand(Side::Right), slot)].into()),
				Equip::new([Item {
					slot: SlotKey::Hand(Side::Right),
					model: "model key".into(),
				}]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Equip>());
	}

	#[test]
	fn remove_equip_component_when_no_matching_model() {
		let model_handle = Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 42));
		let models = Models([(Cow::from("model key"), model_handle.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app
			.world
			.spawn(Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 11)))
			.id();
		let agent = app
			.world
			.spawn((
				Slots([(SlotKey::Hand(Side::Right), slot)].into()),
				Equip::new([Item {
					slot: SlotKey::Hand(Side::Right),
					model: "non matching model key".into(),
				}]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Equip>());
	}

	#[test]
	fn do_not_remove_equip_component_when_no_matching_slot() {
		let model_handle = Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 42));
		let models = Models([(Cow::from("model key"), model_handle.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app
			.world
			.spawn(Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 11)))
			.id();
		let agent = app
			.world
			.spawn((
				Slots([(SlotKey::Hand(Side::Left), slot)].into()),
				Equip::new([Item {
					slot: SlotKey::Hand(Side::Right),
					model: "model key".into(),
				}]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Equip>());
	}

	#[test]
	fn evaluate_equip_per_item() {
		let model_handle = Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 42));
		let models = Models([(Cow::from("model key"), model_handle.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app
			.world
			.spawn(Handle::<Scene>::weak(HandleId::new(Uuid::new_v4(), 11)))
			.id();
		let agent = app
			.world
			.spawn((
				Slots([(SlotKey::Hand(Side::Right), slot)].into()),
				Equip::new([
					Item {
						slot: SlotKey::Hand(Side::Right),
						model: "model key".into(),
					},
					Item {
						slot: SlotKey::Legs,
						model: "model key".into(),
					},
				]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let slot_handle = app.world.entity(slot).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);
		let items = agent.get::<Equip>();

		assert_eq!(
			(
				Some(model_handle),
				Some(&Equip::new([Item {
					slot: SlotKey::Legs,
					model: "model key".into(),
				}]))
			),
			(slot_handle.cloned(), items)
		);
	}
}
