use crate::{
	components::{Collection, Item, Slot, SlotKey, Slots},
	errors::{Error, Level},
	resources::Models,
	traits::accessor::Accessor,
};
use bevy::{
	ecs::component::Component,
	prelude::{Commands, Entity, Handle, Mut, Query, Res},
	scene::Scene,
};

pub fn equip_item<
	TContainer: Component,
	TItemAccessor: Accessor<TContainer, (SlotKey, Item), Item> + Send + Sync + 'static,
>(
	mut commands: Commands,
	models: Res<Models>,
	mut agent: Query<(
		Entity,
		&mut Slots,
		&mut Collection<TItemAccessor>,
		&mut TContainer,
	)>,
	mut scene_handles: Query<&mut Handle<Scene>>,
) -> Vec<Result<(), Error>> {
	let mut results = Vec::new();

	for (agent, mut slots, mut equip, mut component) in &mut agent {
		let mut agent = commands.entity(agent);
		let fails = equip_items_to::<TContainer, TItemAccessor>(
			&mut slots,
			&mut scene_handles,
			&mut component,
			&equip,
			&models,
		);
		let mut retry: Vec<TItemAccessor> = vec![];

		update_retries_and_results(fails, &mut retry, &mut results);

		if retry.is_empty() {
			agent.remove::<Collection<TItemAccessor>>();
		} else {
			equip.0 = retry;
		}
	}

	results
}

fn update_retries_and_results<TItemAccessor>(
	fails: Vec<(TItemAccessor, Result<(), Error>)>,
	retry: &mut Vec<TItemAccessor>,
	results: &mut Vec<Result<(), Error>>,
) {
	for (src, result) in fails {
		match &result {
			Err(error) if error.lvl == Level::Warning => {
				retry.push(src);
			}
			_ => {}
		};
		results.push(result);
	}
}

type SourcesWithErrors<TItemAccessor> = Vec<TItemAccessor>;

fn equip_items_to<
	TContainer: Component,
	TItemAccessor: Accessor<TContainer, (SlotKey, Item), Item>,
>(
	slots: &mut Mut<Slots>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
	component: &mut TContainer,
	equip: &Collection<TItemAccessor>,
	models: &Res<Models>,
) -> SourcesWithErrors<(TItemAccessor, Result<(), Error>)> {
	let try_swap_items = |src: &TItemAccessor| {
		let (slot_key, new_item) = src.get_key_and_item(component)?;
		Some(
			match equip_new_and_return_old_item(slots, scene_handles, &(slot_key, new_item), models)
			{
				Err(error) => (src.with_item(Some(new_item), component), Err(error)),
				Ok(old) => (src.with_item(old, component), Ok(())),
			},
		)
	};

	equip.0.iter().filter_map(try_swap_items).collect()
}

fn equip_new_and_return_old_item(
	slots: &mut Mut<Slots>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
	(slot_key, item): &(SlotKey, Item),
	models: &Res<Models>,
) -> Result<Option<Item>, Error> {
	let slot = get_slot(item, slots, slot_key)?;
	let mut slot_handle = get_slot_handle(item, slot.entity, scene_handles)?;
	let model = get_model(item, models)?;

	let original_item = slot.item;
	slot.item = Some(*item);
	*slot_handle = model.clone();

	Ok(original_item)
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
		ecs::system::{In, IntoSystem},
		prelude::{App, Handle, Update},
		scene::Scene,
		utils::{default, Uuid},
	};
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	#[derive(Default, PartialEq, Debug)]
	enum _Type {
		#[default]
		Original,
		Updated,
	}

	#[derive(Default, PartialEq, Debug)]
	struct _Source {
		r#type: _Type,
		slot: SlotKey,
		item: Option<Item>,
	}

	#[derive(Component, PartialEq, Clone, Copy, Debug)]
	pub struct _Container {
		pub name: &'static str,
	}

	#[automock]
	impl Accessor<_Container, (SlotKey, Item), Item> for _Source {
		fn get_key_and_item(&self, _component: &_Container) -> Option<(SlotKey, Item)> {
			let item = self.item?;
			Some((self.slot, item))
		}

		fn with_item(&self, item: Option<Item>, _component: &mut _Container) -> Self {
			Self {
				slot: self.slot,
				item,
				r#type: _Type::Updated,
			}
		}
	}

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
				_Container { name: "my comp" },
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
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Right),
					item: Some(Item {
						name: "Some Item",
						skill: Some(Skill {
							name: "Some Skill",
							..default()
						}),
						model: Some("model key"),
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);

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
	fn call_source_with_item_none_if_current_slot_item_is_none() {
		let mut app = App::new();
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());
		let slot = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let mut mock_source = Mock_Source::new();
		let component = _Container { name: "my comp" };
		let agent = app.world.spawn(component).id();

		mock_source
			.expect_get_key_and_item()
			.times(1)
			.with(eq(component))
			.return_const((
				SlotKey::Hand(Side::Right),
				Item {
					name: "Some Item",
					..default()
				},
			));
		mock_source
			.expect_with_item()
			.times(1)
			.with(eq(None), eq(component))
			.returning(|_, _| Mock_Source::new());
		app.world.insert_resource(models);
		app.world.entity_mut(agent).insert((
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
			Collection::new([mock_source]),
		));

		app.add_systems(
			Update,
			equip_item::<_Container, Mock_Source>.pipe(|_: In<_>| {}),
		);

		app.update();
	}

	#[test]
	fn call_source_with_current_slot_item() {
		let mut app = App::new();
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());
		let slot = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let mut mock_source = Mock_Source::new();
		let component = _Container {
			name: "my component",
		};
		let agent = app.world.spawn(component).id();

		mock_source
			.expect_get_key_and_item()
			.times(1)
			.with(eq(component))
			.return_const((
				SlotKey::Hand(Side::Right),
				Item {
					name: "Some Item",
					..default()
				},
			));
		mock_source
			.expect_with_item()
			.times(1)
			.with(
				eq(Some(Item {
					name: "Current Item",
					..default()
				})),
				eq(component),
			)
			.returning(|_, _| Mock_Source::new());
		app.world.insert_resource(models);
		app.world.entity_mut(agent).insert((
			Slots(
				[(
					SlotKey::Hand(Side::Right),
					Slot {
						entity: slot,
						item: Some(Item {
							name: "Current Item",
							..default()
						}),
					},
				)]
				.into(),
			),
			Collection::new([mock_source]),
		));

		app.add_systems(
			Update,
			equip_item::<_Container, Mock_Source>.pipe(|_: In<_>| {}),
		);

		app.update();
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
				_Container { name: "my comp" },
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
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Right),
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
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);

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
				_Container { name: "my comp" },
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
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Right),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Collection<_Source>>());
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
				_Container { name: "my comp" },
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
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Right),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: None,
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let slot_model = app.world.entity(slot).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);

		assert_eq!(
			(Some(Handle::<Scene>::default()), false),
			(slot_model.cloned(), agent.contains::<Collection<_Source>>())
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
				_Container { name: "my comp" },
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
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Right),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Collection<_Source>>());
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
				_Container { name: "my comp" },
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
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Right),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Collection<_Source>>());
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
				_Container { name: "my comp" },
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
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Right),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Collection::new([_Source {
				slot: SlotKey::Hand(Side::Right),
				item: Some(Item {
					name: "Some Item",
					skill: None,
					model: Some("model key"),
				}),
				r#type: _Type::Updated,
			}])),
			agent.get::<Collection<_Source>>()
		);
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
				_Container {
					name: "my component",
				},
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
					_Source {
						slot: SlotKey::Hand(Side::Right),
						item: Some(Item {
							name: "Some Item",
							skill: None,
							model: Some("model key"),
						}),
						..default()
					},
					_Source {
						slot: SlotKey::Legs,
						item: Some(Item {
							name: "Some Item",
							skill: None,
							model: Some("model key"),
						}),
						..default()
					},
				]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let slot_model = app.world.entity(slot).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);
		let items = agent.get::<Collection<_Source>>();

		assert_eq!(
			(
				Some(model),
				Some(&Collection::new([_Source {
					slot: SlotKey::Legs,
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
					}),
					r#type: _Type::Updated
				}]))
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
				_Container { name: "my comp" },
				Slots([].into()),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Left),
					item: Some(item),
					..default()
				}]),
			))
			.id();

		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);
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
				_Container { name: "my comp" },
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
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Left),
					item: Some(item),
					..default()
				}]),
			))
			.id();

		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);
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
				_Container { name: "my comp" },
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
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Left),
					item: Some(item),
					..default()
				}]),
			))
			.id();

		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&FakeErrorLogMany([scene_handle_error(&item, slot)].into())),
			agent.get::<FakeErrorLogMany>()
		);
	}
}
