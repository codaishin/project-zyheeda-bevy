use crate::{
	components::{Equipment, Item, Slot, SlotKey, Slots},
	errors::{Error, Level},
	resources::Models,
};
use bevy::{
	prelude::{Commands, Entity, Handle, Mut, Query, Res},
	scene::Scene,
};

pub fn equip_item(
	mut commands: Commands,
	models: Res<Models>,
	mut agent: Query<(Entity, &mut Slots, &mut Equipment)>,
	mut scene_handles: Query<&mut Handle<Scene>>,
) -> Vec<Result<(), Error>> {
	let mut results = Vec::new();

	for (agent, mut slots, mut equip) in &mut agent {
		let items_with_warning = equip_items_to(
			&mut slots,
			&mut scene_handles,
			&equip,
			&models,
			&mut results,
		);

		if items_with_warning.is_empty() {
			commands.entity(agent).remove::<Equipment>();
		} else {
			equip.0 = items_with_warning;
		}
	}

	results
}

type ItemsWithWarning = Vec<(SlotKey, Item)>;

fn equip_items_to(
	slots: &mut Mut<Slots>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
	equip: &Equipment,
	models: &Res<Models>,
	results: &mut Vec<Result<(), Error>>,
) -> ItemsWithWarning {
	let try_to_equip = |item| (item, equip_item_to(slots, scene_handles, item, models));
	let item_with_warning = |(item, result): (&(SlotKey, Item), Result<_, _>)| {
		let warning_item_or_none = match is_warning(&result) {
			true => Some(*item),
			false => None,
		};
		results.push(result);

		warning_item_or_none
	};

	equip
		.0
		.iter()
		.map(try_to_equip)
		.filter_map(item_with_warning)
		.collect()
}

fn is_warning(result: &Result<(), Error>) -> bool {
	let Err(error) = result else {
		return false;
	};
	error.lvl == Level::Warning
}

fn equip_item_to(
	slots: &mut Mut<Slots>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
	(slot_key, item): &(SlotKey, Item),
	models: &Res<Models>,
) -> Result<(), Error> {
	let slot = get_slot(item, slots, slot_key)?;
	let mut slot_handle = get_slot_handle(item, slot.entity, scene_handles)?;
	let model = get_model(item, models)?;

	slot.item = Some(*item);
	*slot_handle = model.clone();

	Ok(())
}

fn get_slot<'a>(
	item: &'a Item,
	slots: &'a mut Mut<'_, Slots>,
	slot_key: &'a SlotKey,
) -> Result<&'a mut Slot, Error> {
	match slots.0.get_mut(slot_key) {
		None => Err(slot_warning(item, *slot_key)),
		Some(slot) => Ok(slot),
	}
}

fn get_slot_handle<'a>(
	item: &Item,
	slot: Entity,
	scene_handles: &'a mut Query<&mut Handle<Scene>>,
) -> Result<Mut<'a, Handle<Scene>>, Error> {
	match scene_handles.get_mut(slot) {
		Err(_) => Err(scene_handle_error(item, slot)),
		Ok(slot_model) => Ok(slot_model),
	}
}

fn get_model(item: &Item, models: &Res<Models>) -> Result<Handle<Scene>, Error> {
	let Some(model_key) = item.model else {
		return Ok(Handle::default());
	};

	let Some(model) = models.0.get(model_key) else {
		return Err(model_error(item, model_key));
	};

	Ok(model.clone())
}

fn slot_warning(item: &Item, slot: SlotKey) -> Error {
	Error {
		msg: format!("{}: slot {:?} not found, retrying next update", item, slot),
		lvl: Level::Warning,
	}
}

fn model_error(item: &Item, model_key: &str) -> Error {
	Error {
		msg: format!("{}: no model found for {}, abandoning", item, model_key),
		lvl: Level::Error,
	}
}

fn scene_handle_error(item: &Item, slot: Entity) -> Error {
	Error {
		msg: format!("{}: {:?} has no Handle<Scene>, abandoning", item, slot),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Cast, Collection, Item, Side, Skill, Slot, SlotKey, Slots},
		resources::Models,
		systems::log::tests::{fake_log_error_lazy_many, FakeErrorLogMany},
	};
	use bevy::{
		asset::AssetId,
		ecs::system::IntoSystem,
		prelude::{App, Handle, Update},
		scene::Scene,
		utils::{default, Uuid},
	};
	use std::time::Duration;

	#[test]
	fn equip_when_marked_to_equip() {
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
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						name: "Some Item",
						skill: Some(Skill {
							name: "Some Skill",
							..default()
						}),
						model: Some("model key"),
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));

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
					item: Some(Item {
						name: "Some Item",
						skill: Some(Skill {
							name: "Some Skill",
							..default()
						}),
						model: Some("model key"),
					}),
				}
			),
			(slot_model.cloned(), slot_component)
		);
	}

	#[test]
	fn equip_when_marked_to_equip_but_no_model_key_set() {
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
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						name: "Some Item",
						skill: Some(Skill {
							cast: Cast {
								pre: Duration::from_millis(1),
								after: Duration::from_millis(2),
							},
							..default()
						}),
						model: None,
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));

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
				Some(Handle::default()),
				&Slot {
					entity: slot,
					item: Some(Item {
						name: "Some Item",
						skill: Some(Skill {
							cast: Cast {
								pre: Duration::from_millis(1),
								after: Duration::from_millis(2),
							},
							..default()
						}),
						model: None,
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
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));

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
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						name: "Some Item",
						skill: None,
						model: None,
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));

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
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));

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
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));

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
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([(
					SlotKey::Hand(Side::Right),
					Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
					},
				)]),
			))
			.id();
		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));

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
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([
					(
						SlotKey::Hand(Side::Right),
						Item {
							name: "Some Item",
							skill: None,
							model: Some("model key"),
						},
					),
					(
						SlotKey::Legs,
						Item {
							name: "Some Item",
							skill: None,
							model: Some("model key"),
						},
					),
				]),
			))
			.id();
		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));

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
						name: "Some Item",
						skill: None,
						model: Some("model key"),
					}
				),]))
			),
			(slot_model.cloned(), items)
		);
	}

	#[test]
	fn return_slot_warning() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());
		let item = Item {
			name: "Some Item",
			skill: Some(Skill {
				name: "Some Skill",
				..default()
			}),
			model: Some("model key"),
		};

		let mut app = App::new();
		app.world.insert_resource(models);
		let agent = app
			.world
			.spawn((
				Slots([].into()),
				Collection::new([(SlotKey::Hand(Side::Left), item)]),
			))
			.id();

		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&FakeErrorLogMany(
				[slot_warning(&item, SlotKey::Hand(Side::Left))].into()
			)),
			agent.get::<FakeErrorLogMany>()
		);
	}

	#[test]
	fn return_model_error() {
		let models = Models([].into());
		let item = Item {
			name: "Some Item",
			skill: Some(Skill {
				name: "Some Skill",
				..default()
			}),
			model: Some("model key"),
		};

		let mut app = App::new();
		let slot = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		app.world.insert_resource(models);
		let agent = app
			.world
			.spawn((
				Slots(
					[(
						SlotKey::Hand(Side::Left),
						Slot {
							entity: slot,
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([(SlotKey::Hand(Side::Left), item)]),
			))
			.id();

		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&FakeErrorLogMany([model_error(&item, "model key")].into())),
			agent.get::<FakeErrorLogMany>()
		);
	}

	#[test]
	fn return_scene_handle_error() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());
		let item = Item {
			name: "Some Item",
			skill: Some(Skill {
				name: "Some Skill",
				..default()
			}),
			model: Some("model key"),
		};

		let mut app = App::new();
		app.world.insert_resource(models);
		let slot = app.world.spawn(()).id();
		let agent = app
			.world
			.spawn((
				Slots(
					[(
						SlotKey::Hand(Side::Left),
						Slot {
							entity: slot,
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([(SlotKey::Hand(Side::Left), item)]),
			))
			.id();

		app.add_systems(Update, equip_item.pipe(fake_log_error_lazy_many(agent)));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&FakeErrorLogMany([scene_handle_error(&item, slot)].into())),
			agent.get::<FakeErrorLogMany>()
		);
	}
}
