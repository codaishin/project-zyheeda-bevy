use crate::{
	components::{Equip, Item, Slot, SlotKey, Slots},
	resources::Models,
};
use bevy::{
	prelude::{Commands, Entity, Handle, Mut, Query, Res},
	scene::Scene,
};
use std::{borrow::Cow, fmt::Debug};
use tracing::{error, info};

enum NoMatch {
	Slot(SlotKey),
	SceneHandle(Entity),
	Model(Cow<'static, str>),
}

type ShouldRetry = bool;
type ItemsToRetry<TBehavior> = Vec<Item<TBehavior>>;

const DO_NOT_RETRY: ShouldRetry = false;
const DONE: ShouldRetry = false;
const RETRY: ShouldRetry = true;

type SlotModel<'a> = Mut<'a, Handle<Scene>>;
type ItemModel = Handle<Scene>;

fn set_slot<TBehavior: Debug>(
	slot_and_model: Result<(&mut Slot<TBehavior>, SlotModel, ItemModel), NoMatch>,
	item: &Item<TBehavior>,
) -> ShouldRetry {
	match slot_and_model {
		Ok((slot, mut slot_model, item_model)) => {
			*slot_model = item_model;
			slot.get_behavior = item.get_behavior;
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

fn equip_item_to<TBehavior: Debug>(
	slots: &mut Slots<TBehavior>,
	item: &Item<TBehavior>,
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

fn equip_items_to<TBehavior: Clone + Debug>(
	slots: &mut Mut<Slots<TBehavior>>,
	equip: &Equip<TBehavior>,
	models: &Res<Models>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
) -> ItemsToRetry<TBehavior> {
	equip
		.0
		.iter()
		.filter(|item| equip_item_to(slots, item, models, scene_handles))
		.cloned()
		.collect()
}

pub fn equip_items<TBehavior: Clone + Debug + 'static>(
	mut commands: Commands,
	models: Res<Models>,
	mut agent: Query<(Entity, &mut Slots<TBehavior>, &mut Equip<TBehavior>)>,
	mut scene_handles: Query<&mut Handle<Scene>>,
) {
	for (agent, mut slots, mut equip) in &mut agent {
		let items_to_retry = equip_items_to(&mut slots, &equip, &models, &mut scene_handles);
		if items_to_retry.is_empty() {
			commands.entity(agent).remove::<Equip<TBehavior>>();
		} else {
			equip.0 = items_to_retry;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Item, Side, Slot, SlotKey, Slots},
		resources::Models,
	};
	use bevy::{
		asset::AssetId,
		prelude::{App, Handle, Ray, Update},
		scene::Scene,
		utils::Uuid,
	};
	use std::borrow::Cow;

	#[derive(Debug, Clone, PartialEq)]
	struct MockBehavior;

	#[test]
	fn equip_when_marked_to_equip() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([(Cow::from("model key"), model.clone())].into());

		fn mock_behavior(_: Ray) -> Option<MockBehavior> {
			Some(MockBehavior)
		}

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
						Slot::<MockBehavior> {
							entity: slot,
							get_behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item::<MockBehavior> {
					get_behavior: Some(mock_behavior),
					slot: SlotKey::Hand(Side::Right),
					model: Some("model key".into()),
				}]),
			))
			.id();
		app.add_systems(Update, equip_items::<MockBehavior>);

		app.update();

		let slot_model = app.world.entity(slot).get::<Handle<Scene>>();
		let slot_component = app
			.world
			.entity(agent)
			.get::<Slots<MockBehavior>>()
			.unwrap()
			.0
			.get(&SlotKey::Hand(Side::Right))
			.unwrap();

		assert_eq!(
			(
				Some(model),
				&Slot {
					entity: slot,
					get_behavior: Some(mock_behavior)
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
						Slot::<MockBehavior> {
							entity: slot,
							get_behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item::<MockBehavior> {
					get_behavior: None,
					slot: SlotKey::Hand(Side::Right),
					model: Some("model key".into()),
				}]),
			))
			.id();
		app.add_systems(Update, equip_items::<MockBehavior>);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Equip<MockBehavior>>());
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
						Slot::<MockBehavior> {
							entity: slot,
							get_behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item::<MockBehavior> {
					get_behavior: None,
					slot: SlotKey::Hand(Side::Right),
					model: None,
				}]),
			))
			.id();
		app.add_systems(Update, equip_items::<MockBehavior>);

		app.update();

		let slot_model = app.world.entity(slot).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);

		assert_eq!(
			(Some(Handle::<Scene>::default()), false),
			(slot_model.cloned(), agent.contains::<Equip<MockBehavior>>())
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
						Slot::<MockBehavior> {
							entity: slot,
							get_behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item::<MockBehavior> {
					get_behavior: None,
					slot: SlotKey::Hand(Side::Right),
					model: Some("model key".into()),
				}]),
			))
			.id();
		app.add_systems(Update, equip_items::<MockBehavior>);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Equip<MockBehavior>>());
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
						Slot::<MockBehavior> {
							entity: slot,
							get_behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item::<MockBehavior> {
					get_behavior: None,
					slot: SlotKey::Hand(Side::Right),
					model: Some("non matching model key".into()),
				}]),
			))
			.id();
		app.add_systems(Update, equip_items::<MockBehavior>);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Equip<MockBehavior>>());
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
						Slot::<MockBehavior> {
							entity: slot,
							get_behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([Item::<MockBehavior> {
					get_behavior: None,
					slot: SlotKey::Hand(Side::Right),
					model: Some("model key".into()),
				}]),
			))
			.id();
		app.add_systems(Update, equip_items::<MockBehavior>);

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Equip<MockBehavior>>());
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
						Slot::<MockBehavior> {
							entity: slot,
							get_behavior: None,
						},
					)]
					.into(),
				),
				Equip::new([
					Item::<MockBehavior> {
						get_behavior: None,
						slot: SlotKey::Hand(Side::Right),
						model: Some("model key".into()),
					},
					Item::<MockBehavior> {
						get_behavior: None,
						slot: SlotKey::Legs,
						model: Some("model key".into()),
					},
				]),
			))
			.id();
		app.add_systems(Update, equip_items::<MockBehavior>);

		app.update();

		let slot_model = app.world.entity(slot).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);
		let items = agent.get::<Equip<MockBehavior>>();

		assert_eq!(
			(
				Some(model),
				Some(&Equip::new([Item::<MockBehavior> {
					get_behavior: None,
					slot: SlotKey::Legs,
					model: Some("model key".into()),
				}]))
			),
			(slot_model.cloned(), items)
		);
	}
}
