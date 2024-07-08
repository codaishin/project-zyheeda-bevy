use crate::{
	components::{slots::Slots, Slot},
	items::{slot_key::SlotKey, Item, Mount},
	skills::Skill,
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

type Components<'a, TItemAccessor, TContainer> = (
	Entity,
	&'a mut Slots<Handle<Skill>>,
	&'a mut Collection<TItemAccessor>,
	&'a mut TContainer,
);

pub fn equip_item<
	TContainer: Component,
	TItemAccessor: Accessor<TContainer, (SlotKey, Option<Item<Handle<Skill>>>), Item<Handle<Skill>>>
		+ Send
		+ Sync
		+ 'static,
>(
	mut commands: Commands,
	models: Res<Models>,
	mut agent: Query<Components<TItemAccessor, TContainer>>,
	mut scene_handles: Query<&mut Handle<Scene>>,
) -> Vec<Result<(), Error>> {
	let mut results = vec![];

	for (agent, mut slots, mut accessors, mut container) in &mut agent {
		let accessors_and_results = equip_items::<TItemAccessor, TContainer>(
			&mut slots,
			&mut scene_handles,
			&mut container,
			&accessors,
			&models,
		);
		let mut retry = vec![];

		push_retries_and_results(accessors_and_results, &mut retry, &mut results);

		if retry.is_empty() {
			commands.try_remove_from::<Collection<TItemAccessor>>(agent);
		} else {
			accessors.0 = retry;
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
	TItemAccessor: Accessor<TContainer, (SlotKey, Option<Item<Handle<Skill>>>), Item<Handle<Skill>>>,
	TContainer: Component,
>(
	slots: &mut Mut<Slots<Handle<Skill>>>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
	container: &mut TContainer,
	accessors: &Collection<TItemAccessor>,
	models: &Res<Models>,
) -> Vec<(TItemAccessor, Result<(), Error>)> {
	let try_swap_items = |accessor: &TItemAccessor| {
		let (slot_key, acc_item) = accessor.get_key_and_item(container);
		match equip_and_return_old(slots, scene_handles, (slot_key, acc_item.as_ref()), models) {
			Ok(old_item) => (accessor.with_item(old_item, container), Ok(())),
			Err(error) => (accessor.with_item(acc_item, container), Err(error)),
		}
	};

	accessors.0.iter().map(try_swap_items).collect()
}

fn equip_and_return_old(
	slots: &mut Mut<Slots<Handle<Skill>>>,
	scene_handles: &mut Query<&mut Handle<Scene>>,
	(slot_key, item): (SlotKey, Option<&Item<Handle<Skill>>>),
	models: &Res<Models>,
) -> Result<Option<Item<Handle<Skill>>>, Error> {
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
	item: Option<&Item<Handle<Skill>>>,
	slots: &'a mut Mut<'_, Slots<Handle<Skill>>>,
	slot_key: SlotKey,
) -> Result<&'a mut Slot<Handle<Skill>>, Error> {
	match slots.0.get_mut(&slot_key) {
		None => Err(slot_warning(item, slot_key)),
		Some(slot) => Ok(slot),
	}
}

fn get_model(
	item: Option<&Item<Handle<Skill>>>,
	models: &Res<Models>,
) -> Result<Handle<Scene>, Error> {
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
	slot: &Slot<Handle<Skill>>,
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

fn swap_and_return_old(
	item: Option<&Item<Handle<Skill>>>,
	slot: &mut Slot<Handle<Skill>>,
) -> Result<Option<Item<Handle<Skill>>>, Error> {
	let mut item = item.cloned();
	swap(&mut item, &mut slot.item);

	Ok(item)
}

fn slot_warning(item: Option<&Item<Handle<Skill>>>, slot: SlotKey) -> Error {
	Error {
		msg: format!(
			"{:#?}: slot::<Handle<Skill>> {:#?} not found, retrying next update",
			item, slot
		),
		lvl: Level::Warning,
	}
}

fn model_error(item: &Item<Handle<Skill>>, model_key: &str) -> Error {
	Error {
		msg: format!("{}: no model found for {}, abandoning", item, model_key),
		lvl: Level::Error,
	}
}

fn scene_handle_error(item: Option<&Item<Handle<Skill>>>, error: QueryEntityError) -> Error {
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

	#[derive(Default, PartialEq, Debug)]
	enum _Type {
		#[default]
		Original,
		Updated,
	}

	#[derive(Default, PartialEq, Debug)]
	struct _Accessor {
		r#type: _Type,
		slot: SlotKey,
		item: Option<Item<Handle<Skill>>>,
	}

	#[derive(Component, PartialEq, Clone, Copy, Debug)]
	pub struct _Container {
		pub name: &'static str,
	}

	#[automock]
	impl Accessor<_Container, (SlotKey, Option<Item<Handle<Skill>>>), Item<Handle<Skill>>>
		for _Accessor
	{
		fn get_key_and_item(
			&self,
			_component: &_Container,
		) -> (SlotKey, Option<Item<Handle<Skill>>>) {
			(self.slot, self.item.clone())
		}

		fn with_item(
			&self,
			item: Option<Item<Handle<Skill>>>,
			_component: &mut _Container,
		) -> Self {
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
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
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let slot_model = app.world.entity(hand).get::<Handle<Scene>>();
		let slot_component = app
			.world
			.entity(agent)
			.get::<Slots<Handle<Skill>>>()
			.expect("no slots")
			.0
			.get(&SlotKey::Hand(Side::Main))
			.expect("nothing in hand");

		assert_eq!(
			(
				Some(model),
				&Slot {
					mounts: Mounts { hand, forearm },
					item: Some(Item {
						name: "Some Item",
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { forearm, hand },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
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
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let slot_model = app.world.entity(forearm).get::<Handle<Scene>>();
		let slot_component = app
			.world
			.entity(agent)
			.get::<Slots<Handle<Skill>>>()
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: Some(Item {
								name: "Some Item",
								model: Some("model key"),
								..default()
							}),
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Main),
					item: None,
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let slot_models = [
			app.world.entity(hand).get::<Handle<Scene>>(),
			app.world.entity(forearm).get::<Handle<Scene>>(),
		];
		let slot_component = app
			.world
			.entity(agent)
			.get::<Slots<Handle<Skill>>>()
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
		let mut mock_accessor = Mock_Accessor::new();
		let component = _Container { name: "my comp" };
		let agent = app.world.spawn(component).id();

		mock_accessor
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
		mock_accessor
			.expect_with_item()
			.times(1)
			.with(eq(None), eq(component))
			.returning(|_, _| Mock_Accessor::new());
		app.world.insert_resource(models);
		app.world.entity_mut(agent).insert((
			Slots::<Handle<Skill>>(
				[(
					SlotKey::Hand(Side::Main),
					Slot {
						mounts: Mounts { hand, forearm },
						item: None,
					},
				)]
				.into(),
			),
			Collection::new([mock_accessor]),
		));

		app.add_systems(
			Update,
			equip_item::<_Container, Mock_Accessor>.pipe(|_: In<_>| {}),
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
		let mut mock_accessor = Mock_Accessor::new();
		let component = _Container {
			name: "my component",
		};
		let agent = app.world.spawn(component).id();

		mock_accessor
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
		mock_accessor
			.expect_with_item()
			.times(1)
			.with(
				eq(Some(Item {
					name: "Current Item",
					..default()
				})),
				eq(component),
			)
			.returning(|_, _| Mock_Accessor::new());
		app.world.insert_resource(models);
		app.world.entity_mut(agent).insert((
			Slots::<Handle<Skill>>(
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
			Collection::new([mock_accessor]),
		));

		app.add_systems(
			Update,
			equip_item::<_Container, Mock_Accessor>.pipe(|_: In<_>| {}),
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						model: None,
						..default()
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let slot_model = app.world.entity(hand).get::<Handle<Scene>>();
		let slot_component = app
			.world
			.entity(agent)
			.get::<Slots<Handle<Skill>>>()
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						model: Some("model key"),
						..default()
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Collection<_Accessor>>());
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						model: None,
						..default()
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let slot_model = app.world.entity(hand).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);

		assert_eq!(
			(Some(Handle::<Scene>::default()), false),
			(
				slot_model.cloned(),
				agent.contains::<Collection<_Accessor>>()
			)
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						model: Some("model key"),
						..default()
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Collection<_Accessor>>());
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Main),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						model: Some("model key"),
						..default()
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Collection<_Accessor>>());
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Off),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Main),
					item: Some(Item {
						name: "Some Item",
						model: Some("model key"),
						..default()
					}),
					..default()
				}]),
			))
			.id();
		app.add_systems(
			Update,
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Collection::new([_Accessor {
				slot: SlotKey::Hand(Side::Main),
				item: Some(Item {
					name: "Some Item",
					model: Some("model key"),
					..default()
				}),
				r#type: _Type::Updated,
			}])),
			agent.get::<Collection<_Accessor>>()
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
				Slots::<Handle<Skill>>(
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
					_Accessor {
						slot: SlotKey::Hand(Side::Main),
						item: Some(Item {
							name: "Some Item",
							model: Some("model key"),
							..default()
						}),
						..default()
					},
					_Accessor {
						slot: SlotKey::Hand(Side::Off),
						item: Some(Item {
							name: "Some Item",
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
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
		);

		app.update();

		let slot_model = app.world.entity(hand).get::<Handle<Scene>>();
		let agent = app.world.entity(agent);
		let items = agent.get::<Collection<_Accessor>>();

		assert_eq!(
			(
				Some(model),
				Some(&Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Off),
					item: Some(Item {
						name: "Some Item",
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
			model: Some("model key"),
			..default()
		};

		let mut app = App::new();
		app.world.insert_resource(models);
		let agent = app
			.world
			.spawn((
				_Container { name: "my comp" },
				Slots::<Handle<Skill>>([].into()),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Off),
					item: Some(item.clone()),
					..default()
				}]),
			))
			.id();

		app.add_systems(
			Update,
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Off),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Off),
					item: Some(item.clone()),
					..default()
				}]),
			))
			.id();

		app.add_systems(
			Update,
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
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
				Slots::<Handle<Skill>>(
					[(
						SlotKey::Hand(Side::Off),
						Slot {
							mounts: Mounts { hand, forearm },
							item: None,
						},
					)]
					.into(),
				),
				Collection::new([_Accessor {
					slot: SlotKey::Hand(Side::Off),
					item: Some(item.clone()),
					..default()
				}]),
			))
			.id();

		app.add_systems(
			Update,
			equip_item::<_Container, _Accessor>.pipe(fake_log_error_lazy_many(agent)),
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
