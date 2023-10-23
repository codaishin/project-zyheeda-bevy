use std::borrow::Cow;

use crate::{
	components::{Equip, SlotKey, Slots},
	resources::Models,
};
use bevy::{
	prelude::{error, info, Commands, Entity, Handle, Query, Res},
	scene::Scene,
};

enum NoMatching {
	Slot(SlotKey),
	SceneHandle(Entity),
	Model(Cow<'static, str>),
}

pub fn equip_item(
	mut commands: Commands,
	models: Res<Models>,
	agent: Query<(Entity, &Slots, &Equip)>,
	mut scene_handles: Query<&mut Handle<Scene>>,
) {
	for (agent, slots, equip) in &agent {
		let handles = slots
			.0
			.get(&equip.slot)
			.ok_or(NoMatching::Slot(equip.slot))
			.and_then(|slot| match scene_handles.get_mut(*slot) {
				Ok(slot_handle) => Ok(slot_handle),
				Err(_) => Err(NoMatching::SceneHandle(*slot)),
			})
			.and_then(|slot_handle| match models.0.get(&equip.model) {
				Some(model_handle) => Ok((slot_handle, model_handle)),
				None => Err(NoMatching::Model(equip.model.clone())),
			});

		match handles {
			Ok((mut slot_handle, model_handle)) => {
				*slot_handle = model_handle.clone();
				commands.entity(agent).remove::<Equip>();
			}
			Err(NoMatching::Model(model_key)) => {
				commands.entity(agent).remove::<Equip>();
				error!(
					"{:?}: no model found for {:?}, abandoning",
					equip, model_key,
				);
			}
			Err(NoMatching::Slot(slot)) => {
				info!(
					"{:?}: slot {:?} not found, retrying next update",
					equip, slot,
				)
			}
			Err(NoMatching::SceneHandle(slot)) => {
				commands.entity(agent).remove::<Equip>();
				error!("{:?}: {:?} has no Handle<Scene>, abandoning", equip, slot);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Equip, Side, SlotKey, Slots},
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
			Equip {
				slot: SlotKey::Hand(Side::Right),
				model: "model key".into(),
			},
		));
		app.add_systems(Update, equip_item);

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
				Equip {
					slot: SlotKey::Hand(Side::Right),
					model: "model key".into(),
				},
			))
			.id();
		app.add_systems(Update, equip_item);

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
				Equip {
					slot: SlotKey::Hand(Side::Right),
					model: "model key".into(),
				},
			))
			.id();
		app.add_systems(Update, equip_item);

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
				Equip {
					slot: SlotKey::Hand(Side::Right),
					model: "non matching model key".into(),
				},
			))
			.id();
		app.add_systems(Update, equip_item);

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
				Equip {
					slot: SlotKey::Hand(Side::Right),
					model: "model key".into(),
				},
			))
			.id();
		app.add_systems(Update, equip_item);

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Equip>());
	}
}
