use crate::{
	components::{Equipment, Item, Slot, SlotKey, Slots},
	resources::Models,
};
use bevy::{
	prelude::{Commands, Entity, Handle, Mut, Query, Res},
	scene::Scene,
};
use tracing::{error, info};

enum NoMatch {
	Slot(SlotKey),
	SceneHandle(Entity),
	Model(&'static str),
}

type ShouldRetry = bool;
type ItemsToRetry = Vec<(SlotKey, Item)>;

const DO_NOT_RETRY: ShouldRetry = false;
const DONE: ShouldRetry = false;
const RETRY: ShouldRetry = true;

type SlotModel<'a> = Mut<'a, Handle<Scene>>;
type ItemModel = Handle<Scene>;

fn set_slot(
	slot_and_model: Result<(&mut Slot, SlotModel, ItemModel), NoMatch>,
	item: &Item,
) -> ShouldRetry {
	match slot_and_model {
		Ok((slot, mut slot_model, item_model)) => {
			*slot_model = item_model;
			slot.skill = item.skill.clone();
			DONE
		}
		Err(NoMatch::Slot(slot)) => {
			info!(
				"{:?}: slot {:?} not found, retrying next update",
				item, slot,
			);
			RETRY
		}
		Err(NoMatch::Model(model_key)) => {
			error!("{:?}: no model found for {:?}, abandoning", item, model_key);
			DO_NOT_RETRY
		}
		Err(NoMatch::SceneHandle(slot)) => {
			error!("{:?}: {:?} has no Handle<Scene>, abandoning", item, slot);
			DO_NOT_RETRY
		}
	}
}

fn equip_item_to(
	slots: &mut Slots,
	(slot, item): &(SlotKey, Item),
	models: &Res<Models>,
	scene_models: &mut Query<&mut Handle<Scene>>,
) -> ShouldRetry {
	let slot_and_model = slots
		.0
		.get_mut(slot)
		.ok_or(NoMatch::Slot(*slot))
		.and_then(|slot| match scene_models.get_mut(slot.entity) {
			Ok(slot_model) => Ok((slot, slot_model)),
			Err(_) => Err(NoMatch::SceneHandle(slot.entity)),
		})
		.and_then(|(slot, slot_model)| {
			let Some(model) = item.model else {
				return Ok((slot, slot_model, Handle::<Scene>::default()));
			};
			match models.0.get(model) {
				Some(model) => Ok((slot, slot_model, model.clone())),
				None => Err(NoMatch::Model(model)),
			}
		});

	set_slot(slot_and_model, item)
}

fn equip_items_to(
	slots: &mut Mut<Slots>,
	equip: &Equipment,
	models: &Res<Models>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
) -> ItemsToRetry {
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
	mut agent: Query<(Entity, &mut Slots, &mut Equipment)>,
	mut scene_handles: Query<&mut Handle<Scene>>,
) {
	for (agent, mut slots, mut equip) in &mut agent {
		let items_to_retry = equip_items_to(&mut slots, &equip, &models, &mut scene_handles);
		if items_to_retry.is_empty() {
			commands.entity(agent).remove::<Equipment>();
		} else {
			equip.0 = items_to_retry;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behaviors::meta::{Agent, BehaviorMeta, Spawner},
		components::{marker::Marker, Cast, Collection, Item, Side, Skill, Slot, SlotKey, Slots},
		resources::Models,
	};
	use bevy::{
		asset::AssetId,
		prelude::{App, Handle, Ray, Update},
		scene::Scene,
		utils::{default, Uuid},
	};
	use std::time::Duration;

	#[test]
	fn equip_when_marked_to_equip() {
		fn fake_start(_: &mut Commands, _: &Agent, _: &Spawner, _: &Ray) {}
		fn fake_stop(_: &mut Commands, _: &Agent) {}

		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let agent = app
			.world
			.spawn((
				Slots(
					[(
						SlotKey::Hand(Side::Right),
						Slot {
							entity: slot,
							skill: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						skill: Some(Skill {
							cast: Cast {
								pre: Duration::from_millis(1),
								after: Duration::from_millis(2),
							},
							markers: Marker::<u32>::commands(),
							behavior: BehaviorMeta {
								run_fn: Some(fake_start),
								stop_fn: Some(fake_stop),
								transform_fn: None,
							},
							..default()
						}),
						model: Some("model key"),
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let slot_model = app.world.entity(slot).get::<Handle<Scene>>();
		let slot_component = app
			.world
			.entity(agent)
			.get::<Slots>()
			.unwrap()
			.0
			.get(&SlotKey::Hand(Side::Right))
			.unwrap();

		assert_eq!(
			(
				Some(model),
				&Slot {
					entity: slot,
					skill: Some(Skill {
						cast: Cast {
							pre: Duration::from_millis(1),
							after: Duration::from_millis(2),
						},
						markers: Marker::<u32>::commands(),
						behavior: BehaviorMeta {
							run_fn: Some(fake_start),
							stop_fn: Some(fake_stop),
							transform_fn: None,
						},
						..default()
					})
				}
			),
			(slot_model.cloned(), slot_component)
		);
	}

	#[test]
	fn remove_equip_component() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let agent = app
			.world
			.spawn((
				Slots(
					[(
						SlotKey::Hand(Side::Right),
						Slot {
							entity: slot,
							skill: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						skill: None,
						model: Some("model key"),
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Equipment>());
	}

	#[test]
	fn set_default_scene_handle_when_no_model_key() {
		let mut app = App::new();
		app.world.insert_resource(Models([].into()));
		let slot = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let agent = app
			.world
			.spawn((
				Slots(
					[(
						SlotKey::Hand(Side::Right),
						Slot {
							entity: slot,
							skill: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						skill: None,
						model: None,
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let slot_model = app.world.entity(slot).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);

		assert_eq!(
			(Some(Handle::<Scene>::default()), false),
			(slot_model.cloned(), agent.contains::<Equipment>())
		);
	}

	#[test]
	fn remove_equip_component_when_no_slot_scene_handle() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app.world.spawn(()).id();
		let agent = app
			.world
			.spawn((
				Slots(
					[(
						SlotKey::Hand(Side::Right),
						Slot {
							entity: slot,
							skill: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						skill: None,
						model: Some("model key"),
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Equipment>());
	}

	#[test]
	fn remove_equip_component_when_no_matching_model() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let agent = app
			.world
			.spawn((
				Slots(
					[(
						SlotKey::Hand(Side::Right),
						Slot {
							entity: slot,
							skill: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						skill: None,
						model: Some("model key"),
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Equipment>());
	}

	#[test]
	fn do_not_remove_equip_component_when_no_matching_slot() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let agent = app
			.world
			.spawn((
				Slots(
					[(
						SlotKey::Hand(Side::Left),
						Slot {
							entity: slot,
							skill: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						skill: None,
						model: Some("model key"),
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Equipment>());
	}

	#[test]
	fn evaluate_equip_per_item() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let agent = app
			.world
			.spawn((
				Slots(
					[(
						SlotKey::Hand(Side::Right),
						Slot {
							entity: slot,
							skill: None,
						},
					)]
					.into(),
				),
				Collection::new([
					(
						SlotKey::Hand(Side::Right),
						Item {
							skill: None,
							model: Some("model key"),
						},
					),
					(
						SlotKey::Legs,
						Item {
							skill: None,
							model: Some("model key"),
						},
					),
				]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let slot_model = app.world.entity(slot).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);
		let items = agent.get::<Equipment>();

		assert_eq!(
			(
				Some(model),
				Some(&Collection::new([(
					SlotKey::Legs,
					Item {
						skill: None,
						model: Some("model key"),
					}
				),]))
			),
			(slot_model.cloned(), items)
		);
	}
}
