use crate::{
	components::{slots::Slots, Slot},
	items::{Item, Mount, SlotKey},
};
use bevy::{
	ecs::{component::Component, query::QueryEntityError},
	prelude::{Commands, Entity, Handle, Mut, Query, Res},
	scene::Scene,
};
use common::{
	components::Collection,
	errors::{Error, Level},
	resources::Models,
	traits::{accessor::Accessor, try_remove_from::TryRemoveFrom},
};
use std::mem::swap;

pub fn equip_item<
	TContainer: Component,
	TItemAccessor: Accessor<TContainer, (SlotKey, Option<Item>), Item> + Send + Sync + 'static,
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
		let accessors_and_results = equip_items::<TContainer, TItemAccessor>(
			&mut slots,
			&mut scene_handles,
			&mut component,
			&equip,
			&models,
		);
		let mut retry: Vec<TItemAccessor> = vec![];

		push_retries_and_results(accessors_and_results, &mut retry, &mut results);

		if retry.is_empty() {
			commands.try_remove_from::<Collection<TItemAccessor>>(agent);
		} else {
			equip.0 = retry;
		}
	}

	results
}

fn push_retries_and_results<TItemAccessor>(
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

fn equip_items<
	TContainer: Component,
	TItemAccessor: Accessor<TContainer, (SlotKey, Option<Item>), Item>,
>(
	slots: &mut Mut<Slots>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
	component: &mut TContainer,
	equip: &Collection<TItemAccessor>,
	models: &Res<Models>,
) -> Vec<(TItemAccessor, Result<(), Error>)> {
	let try_swap_items = |accessor: &TItemAccessor| {
		let (slot_key, acc_item) = accessor.get_key_and_item(component);
		match equip_and_return_old(slots, scene_handles, (slot_key, acc_item.as_ref()), models) {
			Ok(old_item) => (accessor.with_item(old_item, component), Ok(())),
			Err(error) => (accessor.with_item(acc_item, component), Err(error)),
		}
	};

	equip.0.iter().map(try_swap_items).collect()
}

fn equip_and_return_old(
	slots: &mut Mut<Slots>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
	(slot_key, item): (SlotKey, Option<&Item>),
	models: &Res<Models>,
) -> Result<Option<Item>, Error> {
	let slot = get_slot(item, slots, slot_key)?;
	let item_model = get_model(item, models)?;
	let (hand_model, forearm_model) = match item.map(|item| item.mount) {
		Some(Mount::Hand) => (item_model, Handle::default()),
		Some(Mount::Forearm) => (Handle::default(), item_model),
		None => (Handle::default(), Handle::default()),
	};

	match set_model(slot, scene_handles, hand_model, forearm_model) {
		Ok(()) => swap_and_return_old(item, slot),
		Err(error) => Err(scene_handle_error(item, error)),
	}
}

fn get_slot<'a>(
	item: Option<&Item>,
	slots: &'a mut Mut<'_, Slots>,
	slot_key: SlotKey,
) -> Result<&'a mut Slot, Error> {
	match slots.0.get_mut(&slot_key) {
		None => Err(slot_warning(item, slot_key)),
		Some(slot) => Ok(slot),
	}
}

fn get_model(item: Option<&Item>, models: &Res<Models>) -> Result<Handle<Scene>, Error> {
	let Some(item) = item else {
		return Ok(Handle::default());
	};

	let Some(model_key) = item.model else {
		return Ok(Handle::default());
	};

	let Some(model) = models.0.get(model_key) else {
		return Err(model_error(item, model_key));
	};

	Ok(model.clone())
}

fn set_model(
	slot: &Slot,
	scene_handles: &mut Query<&mut Handle<Scene>>,
	model_hand: Handle<Scene>,
	model_forearm: Handle<Scene>,
) -> Result<(), QueryEntityError> {
	let mut handle = scene_handles.get_mut(slot.mounts.hand)?;
	*handle = model_hand;
	let mut handle = scene_handles.get_mut(slot.mounts.forearm)?;
	*handle = model_forearm;

	Ok(())
}

fn swap_and_return_old(item: Option<&Item>, slot: &mut Slot) -> Result<Option<Item>, Error> {
	let mut item = item.cloned();
	swap(&mut item, &mut slot.item);

	Ok(item)
}

fn slot_warning(item: Option<&Item>, slot: SlotKey) -> Error {
	Error {
		msg: format!(
			"{:#?}: slot {:#?} not found, retrying next update",
			item, slot
		),
		lvl: Level::Warning,
	}
}

fn model_error(item: &Item, model_key: &str) -> Error {
	Error {
		msg: format!("{}: no model found for {}, abandoning", item, model_key),
		lvl: Level::Error,
	}
}

fn scene_handle_error(item: Option<&Item>, error: QueryEntityError) -> Error {
	Error {
		msg: format!("{:#?}: {:#?} has no Handle<Scene>, abandoning", item, error),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::Mounts, items::Mount, skills::Skill};
	use bevy::{
		asset::AssetId,
		ecs::system::{In, IntoSystem},
		prelude::{App, Handle, Update},
		scene::Scene,
		utils::{default, Uuid},
	};
	use common::{
		components::Side,
		systems::log::test_tools::{fake_log_error_lazy_many, FakeErrorLogMany},
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
	impl Accessor<_Container, (SlotKey, Option<Item>), Item> for _Source {
		fn get_key_and_item(&self, _component: &_Container) -> (SlotKey, Option<Item>) {
			(self.slot, self.item.clone())
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
	fn equip_hand_when_marked_to_equip() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						skill: Some(Skill {
							name: "Some Skill",
							..default()
						}),
						model: Some("model key"),
						mount: Mount::Hand,
						..default()
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

		let slot_model = app.world.entity(hand).get::<Handle<Scene>>();
		let slot_component = app
			.world
			.entity(agent)
			.get::<Slots>()
			.unwrap()
			.0
			.get(&SlotKey::Hand(Side::Main))
			.unwrap();

		assert_eq!(
			(
				Some(model),
				&Slot {
					mounts: Mounts { hand, forearm },
					item: Some(Item {
						name: "Some Item",
						skill: Some(Skill {
							name: "Some Skill",
							..default()
						}),
						model: Some("model key"),
						mount: Mount::Hand,
						..default()
					}),
				}
			),
			(slot_model.cloned(), slot_component)
		);
	}

	#[test]
	fn equip_forearm_when_marked_to_equip() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { forearm, hand },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						skill: Some(Skill {
							name: "Some Skill",
							..default()
						}),
						model: Some("model key"),
						mount: Mount::Forearm,
						..default()
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

		let slot_model = app.world.entity(forearm).get::<Handle<Scene>>();
		let slot_component = app
			.world
			.entity(agent)
			.get::<Slots>()
			.unwrap()
			.0
			.get(&SlotKey::Hand(Side::Main))
			.unwrap();

		assert_eq!(
			(
				Some(model),
				&Slot {
					mounts: Mounts { forearm, hand },
					item: Some(Item {
						name: "Some Item",
						skill: Some(Skill {
							name: "Some Skill",
							..default()
						}),
						model: Some("model key"),
						mount: Mount::Forearm,
						..default()
					}),
				}
			),
			(slot_model.cloned(), slot_component)
		);
	}

	#[test]
	fn equip_none_when_marked_to_equip() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());

		let mut app = App::new();
		app.world.insert_resource(models);
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: Some(Item {
								name: "Some Item",
								skill: Some(Skill {
									name: "Some Skill",
									..default()
								}),
								model: Some("model key"),
								..default()
							}),
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Main),
					item: None,
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Source>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let slot_models = [
			app.world.entity(hand).get::<Handle<Scene>>(),
			app.world.entity(forearm).get::<Handle<Scene>>(),
		];
		let slot_component = app
			.world
			.entity(agent)
			.get::<Slots>()
			.unwrap()
			.0
			.get(&SlotKey::Hand(Side::Main))
			.unwrap();

		assert_eq!(
			(
				[Some(&Handle::default()), Some(&Handle::default())],
				&Slot {
					mounts: Mounts { hand, forearm },
					item: None,
				}
			),
			(slot_models, slot_component)
		);
	}

	#[test]
	fn call_source_with_item_none_if_current_slot_item_is_none() {
		let mut app = App::new();
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let models = Models([("model key", model.clone())].into());
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
				SlotKey::Hand(Side::Main),
				Some(Item {
					name: "Some Item",
					..default()
				}),
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
					SlotKey::Hand(Side::Main),
					Slot {
						mounts: Mounts { hand, forearm },
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
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
				SlotKey::Hand(Side::Main),
				Some(Item {
					name: "Some Item",
					..default()
				}),
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
					SlotKey::Hand(Side::Main),
					Slot {
						mounts: Mounts { hand, forearm },
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
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						skill: Some(Skill {
							active: Duration::from_millis(1),
							..default()
						}),
						model: None,
						..default()
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

		let slot_model = app.world.entity(hand).get::<Handle<Scene>>();
		let slot_component = app
			.world
			.entity(agent)
			.get::<Slots>()
			.unwrap()
			.0
			.get(&SlotKey::Hand(Side::Main))
			.unwrap();

		assert_eq!(
			(
				Some(Handle::default()),
				&Slot {
					mounts: Mounts { hand, forearm },
					item: Some(Item {
						name: "Some Item",
						skill: Some(Skill {
							active: Duration::from_millis(1),
							..default()
						}),
						model: None,
						..default()
					}),
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
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
						..default()
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
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: None,
						..default()
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

		let slot_model = app.world.entity(hand).get::<Handle<Scene>>();
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
		let hand = app.world.spawn(()).id();
		let forearm = app.world.spawn(()).id();
		let agent = app
			.world
			.spawn((
				_Container { name: "my comp" },
				Slots(
					[(
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
						..default()
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
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
						..default()
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
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
						SlotKey::Hand(Side::Off),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
						..default()
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
				slot: SlotKey::Hand(Side::Main),
				item: Some(Item {
					name: "Some Item",
					skill: None,
					model: Some("model key"),
					..default()
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
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([
					_Source {
						slot: SlotKey::Hand(Side::Main),
						item: Some(Item {
							name: "Some Item",
							skill: None,
							model: Some("model key"),
							..default()
						}),
						..default()
					},
					_Source {
						slot: SlotKey::Hand(Side::Off),
						item: Some(Item {
							name: "Some Item",
							skill: None,
							model: Some("model key"),
							..default()
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

		let slot_model = app.world.entity(hand).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);
		let items = agent.get::<Collection<_Source>>();

		assert_eq!(
			(
				Some(model),
				Some(&Collection::new([_Source {
					slot: SlotKey::Hand(Side::Off),
					item: Some(Item {
						name: "Some Item",
						skill: None,
						model: Some("model key"),
						..default()
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
			..default()
		};

		let mut app = App::new();
		app.world.insert_resource(models);
		let agent = app
			.world
			.spawn((
				_Container { name: "my comp" },
				Slots([].into()),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Off),
					item: Some(item.clone()),
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
				[slot_warning(Some(&item), SlotKey::Hand(Side::Off))].into()
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
			..default()
		};

		let mut app = App::new();
		let hand = app
			.world
			.spawn(Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}))
			.id();
		let forearm = app
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
						SlotKey::Hand(Side::Off),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Off),
					item: Some(item.clone()),
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
			..default()
		};

		let mut app = App::new();
		app.world.insert_resource(models);
		let hand = app.world.spawn(()).id();
		let forearm = app.world.spawn(()).id();
		let agent = app
			.world
			.spawn((
				_Container { name: "my comp" },
				Slots(
					[(
						SlotKey::Hand(Side::Off),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Source {
					slot: SlotKey::Hand(Side::Off),
					item: Some(item.clone()),
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
				[scene_handle_error(
					Some(&item),
					QueryEntityError::QueryDoesNotMatch(hand)
				)]
				.into()
			)),
			agent.get::<FakeErrorLogMany>()
		);
	}
}
