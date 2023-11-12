use crate::{
	components::{Equip, Item, Slot, SlotKey, Slots},
	resources::Models,
};
use bevy::{
	prelude::{Commands, Entity, Handle, Mut, Query, Res},
	scene::Scene,
};
use std::borrow::Cow;
use tracing::{error, info};

enum NoMatch {
	Slot(SlotKey),
	SceneHandle(Entity),
	Model(Cow<'static, str>),
}

type ShouldRetry = bool;
type ItemsToRetry = Vec<Item>;

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
			slot.behavior = item.behavior;
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
	item: &Item,
	models: &Res<Models>,
	scene_models: &mut Query<&mut Handle<Scene>>,
) -> ShouldRetry {
	let slot_and_model = slots
		.0
		.get_mut(&item.slot)
		.ok_or(NoMatch::Slot(item.slot))
		.and_then(|slot| match scene_models.get_mut(slot.entity) {
			Ok(slot_model) => Ok((slot, slot_model)),
			Err(_) => Err(NoMatch::SceneHandle(slot.entity)),
		})
		.and_then(|(slot, slot_model)| {
			let Some(model) = item.model.clone() else {
				return Ok((slot, slot_model, Handle::<Scene>::default()));
			};
			match models.0.get(&model) {
				Some(model) => Ok((slot, slot_model, model.clone())),
				None => Err(NoMatch::Model(model)),
			}
		});

	set_slot(slot_and_model, item)
}

fn equip_items_to(
	slots: &mut Mut<Slots>,
	equip: &Equip,
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
	mut agent: Query<(Entity, &mut Slots, &mut Equip)>,
	mut scene_handles: Query<&mut Handle<Scene>>,
) {
	for (agent, mut slots, mut equip) in &mut agent {
		let items_to_retry = equip_items_to(&mut slots, &equip, &models, &mut scene_handles);
		if items_to_retry.is_empty() {
			commands.entity(agent).remove::<Equip>();
		} else {
			equip.0 = items_to_retry;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behaviors::Behavior,
		components::{Item, Side, Slot, SlotKey, Slots},
		resources::Models,
	};
	use bevy::{
		asset::AssetId,
		ecs::system::EntityCommands,
		prelude::{App, Handle, Ray, Update},
		scene::Scene,
		utils::Uuid,
	};
	use std::borrow::Cow;

	fn fake_behavior_insert<const T: char>(_entity: &mut EntityCommands, _ray: Ray) {}

	#[test]
	fn equip_when_marked_to_equip() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([(Cow::from("model key"), model.clone())].into());

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
							behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item {
					behavior: Some(Behavior {
						insert_fn: fake_behavior_insert::<'!'>,
					}),
					slot: SlotKey::Hand(Side::Right),
					model: Some("model key".into()),
				}]),
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
					behavior: Some(Behavior {
						insert_fn: fake_behavior_insert::<'!'>,
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
		let models = Models([(Cow::from("model key"), model.clone())].into());

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
							behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item {
					behavior: None,
					slot: SlotKey::Hand(Side::Right),
					model: Some("model key".into()),
				}]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Equip>());
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
							behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item {
					behavior: None,
					slot: SlotKey::Hand(Side::Right),
					model: None,
				}]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let slot_model = app.world.entity(slot).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);

		assert_eq!(
			(Some(Handle::<Scene>::default()), false),
			(slot_model.cloned(), agent.contains::<Equip>())
		);
	}

	#[test]
	fn remove_equip_component_when_no_slot_scene_handle() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([(Cow::from("model key"), model.clone())].into());

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
							behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item {
					behavior: None,
					slot: SlotKey::Hand(Side::Right),
					model: Some("model key".into()),
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
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([(Cow::from("model key"), model.clone())].into());

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
							behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item {
					behavior: None,
					slot: SlotKey::Hand(Side::Right),
					model: Some("non matching model key".into()),
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
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([(Cow::from("model key"), model.clone())].into());

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
							behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item {
					behavior: None,
					slot: SlotKey::Hand(Side::Right),
					model: Some("model key".into()),
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
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([(Cow::from("model key"), model.clone())].into());

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
							behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([
					Item {
						behavior: None,
						slot: SlotKey::Hand(Side::Right),
						model: Some("model key".into()),
					},
					Item {
						behavior: None,
						slot: SlotKey::Legs,
						model: Some("model key".into()),
					},
				]),
			))
			.id();
		app.add_systems(Update, equip_items);

		app.update();

		let slot_model = app.world.entity(slot).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);
		let items = agent.get::<Equip>();

		assert_eq!(
			(
				Some(model),
				Some(&Equip::new([Item {
					behavior: None,
					slot: SlotKey::Legs,
					model: Some("model key".into()),
				}]))
			),
			(slot_model.cloned(), items)
		);
	}
}
