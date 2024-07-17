use crate::{
	components::{slots::Slots, Slot},
	items::{slot_key::SlotKey, Item, Mount},
	skills::Skill,
	traits::swap_commands::SwapController,
};
use bevy::{
	ecs::component::Component,
	prelude::{Commands, Entity, Handle, Query, Res},
	scene::Scene,
};
use common::{
	errors::{Error, Level},
	resources::Models,
	traits::{
		swap_command::{SwapCommands, SwapError, SwapIn, SwappedOut},
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};
use std::mem::swap;

type Components<'a, TContainer, TSwaps> = (
	Entity,
	&'a mut Slots<Handle<Skill>>,
	&'a mut TContainer,
	&'a mut TSwaps,
);

pub fn equip_item<TContainer, TInnerKey, TSwaps>(
	mut commands: Commands,
	mut agent: Query<Components<TContainer, TSwaps>>,
	models: Res<Models>,
) -> Vec<Result<(), Error>>
where
	TContainer: Component,
	TSwaps: Component,
	for<'a> SwapController<'a, TInnerKey, SlotKey, TContainer, TSwaps>:
		SwapCommands<SlotKey, Item<Handle<Skill>>>,
{
	let mut results = vec![];
	let commands = &mut commands;
	let models = &models;

	for (agent, mut slots, mut container, mut swaps) in &mut agent {
		let slots = slots.as_mut();
		let mut swap_commands = SwapController::new(container.as_mut(), swaps.as_mut());

		swap_commands.try_swap(|slot_key, SwapIn(item)| {
			match try_equip(commands, slots, slot_key, item, models) {
				Ok(swapped_out) => Ok(swapped_out),
				Err((swap_error, log_error)) => {
					results.push(Err(log_error));
					Err(swap_error)
				}
			}
		});

		if swap_commands.is_empty() {
			commands.try_remove_from::<TSwaps>(agent);
		}
	}

	results
}

fn try_equip(
	commands: &mut Commands,
	slots: &mut Slots<Handle<Skill>>,
	slot_key: SlotKey,
	item: Option<Item<Handle<Skill>>>,
	models: &Res<Models>,
) -> Result<SwappedOut<Item<Handle<Skill>>>, (SwapError, Error)> {
	let slot = get_slot(slots, slot_key)?;
	let item_model = get_model(&item, models)?;

	let (hand_model, forearm_model) = match item.as_ref().map(|item| item.mount) {
		Some(Mount::Hand) => (item_model, Handle::default()),
		Some(Mount::Forearm) => (Handle::default(), item_model),
		None => (Handle::default(), Handle::default()),
	};

	commands.try_insert_on(slot.mounts.hand, hand_model);
	commands.try_insert_on(slot.mounts.forearm, forearm_model);

	Ok(swap_item(item, slot))
}

fn get_slot(
	slots: &mut Slots<Handle<Skill>>,
	slot_key: SlotKey,
) -> Result<&mut Slot<Handle<Skill>>, (SwapError, Error)> {
	match slots.0.get_mut(&slot_key) {
		Some(slot) => Ok(slot),
		None => Err((SwapError::TryAgain, slot_warning(slot_key))),
	}
}

fn get_model(
	item: &Option<Item<Handle<Skill>>>,
	models: &Res<Models>,
) -> Result<Handle<Scene>, (SwapError, Error)> {
	let Some(item) = item else {
		return Ok(Handle::default());
	};

	let Some(model_key) = item.model else {
		return Ok(Handle::default());
	};

	let Some(model) = models.0.get(model_key) else {
		return Err((SwapError::Disregard, model_error(item)));
	};

	Ok(model.clone())
}

fn swap_item(
	mut item: Option<Item<Handle<Skill>>>,
	slot: &mut Slot<Handle<Skill>>,
) -> SwappedOut<Item<Handle<Skill>>> {
	swap(&mut item, &mut slot.item);

	SwappedOut(item)
}

fn slot_warning(slot: SlotKey) -> Error {
	Error {
		msg: format!("Slot `{:?}` not found, retrying next update", slot),
		lvl: Level::Warning,
	}
}

fn model_error(item: &Item<Handle<Skill>>) -> Error {
	Error {
		msg: format!(
			"Item({}): no model '{:?}' seems to exist, abandoning",
			item.name, item.model
		),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::Mounts, items::Mount, skills::Skill};
	use bevy::{
		asset::{Asset, AssetId},
		ecs::system::IntoSystem,
		prelude::{App, Handle, Update},
		scene::Scene,
		utils::default,
	};
	use common::{
		components::Side,
		systems::log::test_tools::{fake_log_error_many_recourse, FakeErrorLogManyResource},
		test_tools::utils::SingleThreadedApp,
		traits::swap_command::{SwapError, SwapIn, SwapResult, SwappedOut},
	};
	use std::collections::HashMap;
	use uuid::Uuid;

	#[derive(Component, Default, PartialEq, Debug)]
	struct _Swaps {
		is_empty: bool,
	}

	#[derive(Component, PartialEq, Clone, Debug, Default)]
	pub struct _Container {
		swap_ins: HashMap<SlotKey, SwapIn<Item<Handle<Skill>>>>,
		swap_outs: HashMap<SlotKey, SwappedOut<Item<Handle<Skill>>>>,
		errors: HashMap<SlotKey, SwapError>,
	}

	type SkillItem = Item<Handle<Skill>>;

	impl<'a> SwapCommands<SlotKey, SkillItem> for SwapController<'a, (), SlotKey, _Container, _Swaps> {
		fn try_swap(
			&mut self,
			mut swap_fn: impl FnMut(SlotKey, SwapIn<SkillItem>) -> SwapResult<SkillItem>,
		) {
			let SwapController { container, .. } = self;
			for (slot_key, swap_in) in container.swap_ins.clone() {
				match swap_fn(slot_key, swap_in.clone()) {
					Ok(swap_out) => {
						container.swap_outs.insert(slot_key, swap_out);
					}
					Err(error) => {
						container.errors.insert(slot_key, error);
					}
				}
			}
		}

		fn is_empty(&self) -> bool {
			let SwapController { swaps, .. } = self;
			swaps.is_empty
		}
	}

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::<T>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	fn setup(models: Models) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(models);
		app.add_systems(
			Update,
			equip_item::<_Container, (), _Swaps>.pipe(fake_log_error_many_recourse),
		);

		app
	}

	#[test]
	fn set_hand_model_handle() {
		let model = new_handle();
		let models = Models([("model key", model.clone())].into());
		let mut app = setup(models);

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Slots::<Handle<Skill>>::new([(
				SlotKey::Hand(Side::Main),
				Slot {
					item: None,
					mounts: Mounts { hand, forearm },
				},
			)]),
			_Container {
				swap_ins: HashMap::from([(
					SlotKey::Hand(Side::Main),
					SwapIn(Some(Item {
						model: Some("model key"),
						mount: Mount::Hand,
						..default()
					})),
				)]),
				..default()
			},
			_Swaps { is_empty: false },
		));

		app.update();

		let forearm = app.world().entity(forearm);
		let hand = app.world().entity(hand);

		assert_eq!(
			(Some(&model), Some(&default()),),
			(hand.get::<Handle<Scene>>(), forearm.get::<Handle<Scene>>())
		);
	}

	#[test]
	fn set_forearm_model_handle() {
		let model = new_handle();
		let models = Models([("model key", model.clone())].into());
		let mut app = setup(models);

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Slots::<Handle<Skill>>::new([(
				SlotKey::Hand(Side::Main),
				Slot {
					item: None,
					mounts: Mounts { hand, forearm },
				},
			)]),
			_Container {
				swap_ins: HashMap::from([(
					SlotKey::Hand(Side::Main),
					SwapIn(Some(Item {
						model: Some("model key"),
						mount: Mount::Forearm,
						..default()
					})),
				)]),
				..default()
			},
			_Swaps { is_empty: false },
		));

		app.update();

		let forearm = app.world().entity(forearm);
		let hand = app.world().entity(hand);

		assert_eq!(
			(Some(&default()), Some(&model)),
			(hand.get::<Handle<Scene>>(), forearm.get::<Handle<Scene>>())
		);
	}

	#[test]
	fn set_swap_in_item() {
		let mut app = setup(default());

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		let agent = app
			.world_mut()
			.spawn((
				Slots::<Handle<Skill>>::new([(
					SlotKey::Hand(Side::Main),
					Slot {
						mounts: Mounts { hand, forearm },
						item: None,
					},
				)]),
				_Container {
					swap_ins: HashMap::from([(
						SlotKey::Hand(Side::Main),
						SwapIn(Some(Item {
							name: "swap in",
							..default()
						})),
					)]),
					..default()
				},
				_Swaps { is_empty: false },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Slots::<Handle<Skill>>::new([(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: Mounts { hand, forearm },
					item: Some(Item {
						name: "swap in",
						..default()
					}),
				},
			)])),
			agent.get::<Slots<Handle<Skill>>>()
		);
	}

	#[test]
	fn set_swap_out_item() {
		let mut app = setup(default());

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		let agent = app
			.world_mut()
			.spawn((
				Slots::<Handle<Skill>>::new([(
					SlotKey::Hand(Side::Main),
					Slot {
						mounts: Mounts { hand, forearm },
						item: Some(Item {
							name: "swap out",
							..default()
						}),
					},
				)]),
				_Container {
					swap_ins: HashMap::from([(SlotKey::Hand(Side::Main), SwapIn(None))]),
					..default()
				},
				_Swaps { is_empty: false },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);
		let container = agent.get::<_Container>().unwrap();

		assert_eq!(
			HashMap::from([(
				SlotKey::Hand(Side::Main),
				SwappedOut(Some(Item {
					name: "swap out",
					..default()
				}))
			)]),
			container.swap_outs
		);
	}

	#[test]
	fn try_again_error_when_slot_not_found() {
		let mut app = setup(default());

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		let agent = app
			.world_mut()
			.spawn((
				Slots::<Handle<Skill>>::new([(
					SlotKey::Hand(Side::Main),
					Slot {
						item: None,
						mounts: Mounts { hand, forearm },
					},
				)]),
				_Container {
					swap_ins: HashMap::from([(SlotKey::Hand(Side::Off), SwapIn(None))]),
					..default()
				},
				_Swaps { is_empty: false },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);
		let container = agent.get::<_Container>().unwrap();

		assert_eq!(
			HashMap::from([(SlotKey::Hand(Side::Off), SwapError::TryAgain)]),
			container.errors
		);
	}

	#[test]
	fn return_error_when_slot_not_found() {
		let mut app = setup(default());

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Slots::<Handle<Skill>>::new([(
				SlotKey::Hand(Side::Main),
				Slot {
					item: None,
					mounts: Mounts { hand, forearm },
				},
			)]),
			_Container {
				swap_ins: HashMap::from([(SlotKey::Hand(Side::Off), SwapIn(None))]),
				..default()
			},
			_Swaps { is_empty: false },
		));

		app.update();

		let error_log = app.world().get_resource::<FakeErrorLogManyResource>();

		assert_eq!(
			Some(&FakeErrorLogManyResource(vec![slot_warning(
				SlotKey::Hand(Side::Off)
			)])),
			error_log
		);
	}

	#[test]
	fn disregard_error_when_model_not_found() {
		let mut app = setup(default());

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		let agent = app
			.world_mut()
			.spawn((
				Slots::<Handle<Skill>>::new([(
					SlotKey::Hand(Side::Main),
					Slot {
						item: None,
						mounts: Mounts { hand, forearm },
					},
				)]),
				_Container {
					swap_ins: HashMap::from([(
						SlotKey::Hand(Side::Main),
						SwapIn(Some(Item {
							model: Some("this model does not exist"),
							..default()
						})),
					)]),
					..default()
				},
				_Swaps { is_empty: false },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);
		let container = agent.get::<_Container>().unwrap();

		assert_eq!(
			HashMap::from([(SlotKey::Hand(Side::Main), SwapError::Disregard)]),
			container.errors
		);
	}

	#[test]
	fn return_error_when_model_not_found() {
		let mut app = setup(default());

		let hand = app.world_mut().spawn_empty().id();
		let forearm = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			Slots::<Handle<Skill>>::new([(
				SlotKey::Hand(Side::Main),
				Slot {
					item: None,
					mounts: Mounts { hand, forearm },
				},
			)]),
			_Container {
				swap_ins: HashMap::from([(
					SlotKey::Hand(Side::Main),
					SwapIn(Some(Item {
						name: "item with faulty model",
						model: Some("this model does not exist"),
						..default()
					})),
				)]),
				..default()
			},
			_Swaps { is_empty: false },
		));

		app.update();

		let error_log = app.world().get_resource::<FakeErrorLogManyResource>();

		assert_eq!(
			Some(&FakeErrorLogManyResource(vec![model_error(&Item {
				name: "item with faulty model",
				model: Some("this model does not exist"),
				..default()
			})])),
			error_log
		);
	}

	#[test]
	fn remove_swap_component_when_empty() {
		let mut app = setup(default());

		let agent = app
			.world_mut()
			.spawn((
				Slots::<Handle<Skill>>::default(),
				_Container::default(),
				_Swaps { is_empty: true },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Swaps>());
	}

	#[test]
	fn do_not_remove_swap_component_when_not_empty() {
		let mut app = setup(default());

		let agent = app
			.world_mut()
			.spawn((
				Slots::<Handle<Skill>>::default(),
				_Container::default(),
				_Swaps { is_empty: false },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Swaps { is_empty: false }), agent.get::<_Swaps>());
	}
}
