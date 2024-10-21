use bevy::prelude::*;
use common::{
	components::{Collection, Swap},
	errors::{Error, Level},
	traits::{accessors::get::GetRef, try_remove_from::TryRemoveFrom},
};
use skills::{components::slots::Slots, slot_key::SlotKey};

type SlotsToSwap<'a, THandMounts, TForearmMounts> = (
	Entity,
	&'a mut Slots,
	&'a Collection<Swap<SlotKey, SlotKey>>,
	&'a THandMounts,
	&'a TForearmMounts,
);

pub fn swap_equipped_items<THandMounts, TForearmMounts>(
	mut commands: Commands,
	mut slots_to_swap: Query<SlotsToSwap<THandMounts, TForearmMounts>>,
	mut handles: Query<&mut Handle<Scene>>,
) -> Vec<Result<(), Error>>
where
	THandMounts: Component + GetRef<SlotKey, Entity>,
	TForearmMounts: Component + GetRef<SlotKey, Entity>,
{
	let mut results = vec![];

	for (agent, mut slots, swaps, hands, forearms) in &mut slots_to_swap {
		for swap in &swaps.0 {
			let [result0, result1] = do_swap(swap, &mut slots, &mut handles, hands, forearms);
			if result0.is_err() {
				results.push(result0);
			}
			if result1.is_err() {
				results.push(result1);
			}
		}

		commands.try_remove_from::<Collection<Swap<SlotKey, SlotKey>>>(agent);
	}

	results
}

fn do_swap<THandMounts, TForearmMounts>(
	swap: &Swap<SlotKey, SlotKey>,
	slots: &mut Mut<Slots>,
	handles: &mut Query<&mut Handle<Scene>>,
	hands: &THandMounts,
	forearms: &TForearmMounts,
) -> [Result<(), Error>; 2]
where
	THandMounts: Component + GetRef<SlotKey, Entity>,
	TForearmMounts: Component + GetRef<SlotKey, Entity>,
{
	let slot_results = [
		slots.0.get(&swap.0).cloned().ok_or(no_slot(swap.0)),
		slots.0.get(&swap.1).cloned().ok_or(no_slot(swap.1)),
	];

	let [Ok(slot0), Ok(slot1)] = slot_results else {
		return slot_results.map(drop_ok);
	};

	let mounts = [
		get_mounts(&swap.0, handles, hands, forearms).map_err(no_handle(swap.0)),
		get_mounts(&swap.1, handles, hands, forearms).map_err(no_handle(swap.1)),
	];

	let [Ok(mount0), Ok(mount1)] = mounts else {
		return mounts.map(drop_ok);
	};

	if let Some(slot) = slots.0.get_mut(&swap.0) {
		*slot = slot1;
	}
	if let Some(slot) = slots.0.get_mut(&swap.1) {
		*slot = slot0;
	}
	if let Ok(mut handle) = handles.get_mut(mount0.hand.entity) {
		*handle = mount1.hand.handle;
	}
	if let Ok(mut handle) = handles.get_mut(mount0.forearm.entity) {
		*handle = mount1.forearm.handle;
	}
	if let Ok(mut handle) = handles.get_mut(mount1.hand.entity) {
		*handle = mount0.hand.handle;
	}
	if let Ok(mut handle) = handles.get_mut(mount1.forearm.entity) {
		*handle = mount0.forearm.handle;
	}

	[Ok(()), Ok(())]
}

enum GetHandleError {
	EntityUnknown,
	QueryEntityError,
}

struct Mounts {
	hand: Mount,
	forearm: Mount,
}

struct Mount {
	entity: Entity,
	handle: Handle<Scene>,
}

fn get_mounts<THandMounts, TForearmMounts>(
	key: &SlotKey,
	handles: &Query<&mut Handle<Scene>>,
	hands: &THandMounts,
	forearms: &TForearmMounts,
) -> Result<Mounts, GetHandleError>
where
	THandMounts: Component + GetRef<SlotKey, Entity>,
	TForearmMounts: Component + GetRef<SlotKey, Entity>,
{
	let hand = hands.get(key).ok_or(GetHandleError::EntityUnknown)?;
	let forearm = forearms.get(key).ok_or(GetHandleError::EntityUnknown)?;

	Ok(Mounts {
		hand: handles
			.get(*hand)
			.map(|handle| Mount {
				entity: *hand,
				handle: handle.clone(),
			})
			.map_err(|_| GetHandleError::QueryEntityError)?,
		forearm: handles
			.get(*forearm)
			.map(|handle| Mount {
				entity: *forearm,
				handle: handle.clone(),
			})
			.map_err(|_| GetHandleError::QueryEntityError)?,
	})
}

fn drop_ok<V>(result: Result<V, Error>) -> Result<(), Error> {
	match result {
		Err(error) => Err(error),
		Ok(_) => Ok(()),
	}
}

fn no_slot(slot_key: SlotKey) -> Error {
	Error {
		msg: format!("{:?}: Slot not found", slot_key),
		lvl: Level::Error,
	}
}

fn no_handle(slot_key: SlotKey) -> impl Fn(GetHandleError) -> Error {
	move |error| match error {
		GetHandleError::QueryEntityError => handle_error(slot_key),
		GetHandleError::EntityUnknown => handle_entity_error(slot_key),
	}
}

fn handle_error(slot_key: SlotKey) -> Error {
	Error {
		msg: format!("{:?}: Handle not found", slot_key),
		lvl: Level::Error,
	}
}

fn handle_entity_error(slot_key: SlotKey) -> Error {
	Error {
		msg: format!("{:?}: Handle Entity not found", slot_key),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;
	use common::components::Side;
	use skills::{
		item::{item_type::SkillItemType, SkillItem},
		skills::Skill,
	};
	use std::collections::HashMap;
	use uuid::Uuid;

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::<T>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	#[derive(Component, Default)]
	struct _Hands(HashMap<SlotKey, Entity>);

	impl GetRef<SlotKey, Entity> for _Hands {
		fn get(&self, key: &SlotKey) -> Option<&Entity> {
			self.0.get(key)
		}
	}

	#[derive(Component, Default)]
	struct _Forearms(HashMap<SlotKey, Entity>);

	impl GetRef<SlotKey, Entity> for _Forearms {
		fn get(&self, key: &SlotKey) -> Option<&Entity> {
			self.0.get(key)
		}
	}

	#[derive(Clone)]
	struct _Mount {
		entity: Entity,
		handle: Handle<Scene>,
	}

	fn create_mounts<const N: usize>(app: &mut App) -> [_Mount; N] {
		[(); N].map(|_| new_handle()).map(|handle| _Mount {
			entity: app.world_mut().spawn(handle.clone()).id(),
			handle: handle.clone(),
		})
	}

	#[test]
	fn swap_items() {
		let mut app = App::new();
		let mounts = create_mounts::<4>(&mut app);
		let agent = app
			.world_mut()
			.spawn((
				_Hands(
					[
						(SlotKey::BottomHand(Side::Left), mounts[0].entity),
						(SlotKey::BottomHand(Side::Right), mounts[1].entity),
					]
					.into(),
				),
				_Forearms(
					[
						(SlotKey::BottomHand(Side::Left), mounts[2].entity),
						(SlotKey::BottomHand(Side::Right), mounts[3].entity),
					]
					.into(),
				),
				Slots::<Skill>(
					[
						(
							SlotKey::BottomHand(Side::Left),
							Some(SkillItem {
								name: "left item",
								item_type: SkillItemType::Bracer,
								..default()
							}),
						),
						(
							SlotKey::BottomHand(Side::Right),
							Some(SkillItem {
								name: "right item",
								item_type: SkillItemType::Pistol,
								..default()
							}),
						),
					]
					.into(),
				),
				Collection(
					[Swap(
						SlotKey::BottomHand(Side::Left),
						SlotKey::BottomHand(Side::Right),
					)]
					.into(),
				),
			))
			.id();

		let errors = app
			.world_mut()
			.run_system_once(swap_equipped_items::<_Hands, _Forearms>);

		let swapped_handles = mounts.clone().map(|mount| {
			app.world()
				.entity(mount.entity)
				.get::<Handle<Scene>>()
				.unwrap()
		});
		let slots = app.world().entity(agent).get::<Slots>().unwrap();
		let new_items = (
			slots
				.0
				.get(&SlotKey::BottomHand(Side::Left))
				.unwrap()
				.clone(),
			slots
				.0
				.get(&SlotKey::BottomHand(Side::Right))
				.unwrap()
				.clone(),
		);

		assert_eq!(
			(
				[
					&mounts[1].handle,
					&mounts[0].handle,
					&mounts[3].handle,
					&mounts[2].handle,
				],
				(
					Some(SkillItem {
						name: "right item",
						item_type: SkillItemType::Pistol,
						..default()
					}),
					Some(SkillItem {
						name: "left item",
						item_type: SkillItemType::Bracer,
						..default()
					})
				),
				vec![]
			),
			(swapped_handles, new_items, errors)
		);
	}

	#[test]
	fn remove_collection() {
		let mut app = App::new();
		let agent = app
			.world_mut()
			.spawn((
				_Hands::default(),
				_Forearms::default(),
				Slots::<Skill>([].into()),
				Collection::<Swap<SlotKey, SlotKey>>([].into()),
			))
			.id();

		app.world_mut()
			.run_system_once(swap_equipped_items::<_Hands, _Forearms>);

		let agent = app.world().entity(agent);

		assert!(!agent.contains::<Collection<Swap<SlotKey, SlotKey>>>());
	}

	#[test]
	fn log_slot_errors() {
		let mut app = App::new();
		app.world_mut().spawn((
			_Hands::default(),
			_Forearms::default(),
			Slots::<Skill>([].into()),
			Collection(
				[Swap(
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				)]
				.into(),
			),
		));

		let errors = app
			.world_mut()
			.run_system_once(swap_equipped_items::<_Hands, _Forearms>);

		assert_eq!(
			vec![
				Err(no_slot(SlotKey::BottomHand(Side::Left))),
				Err(no_slot(SlotKey::BottomHand(Side::Right))),
			],
			errors
		)
	}

	#[test]
	fn log_handle_entity_errors() {
		let mut app = App::new();
		app.world_mut().spawn((
			_Hands::default(),
			_Forearms::default(),
			Slots::<Skill>(
				[
					(
						SlotKey::BottomHand(Side::Left),
						Some(SkillItem {
							name: "left item",
							..default()
						}),
					),
					(
						SlotKey::BottomHand(Side::Right),
						Some(SkillItem {
							name: "right item",
							..default()
						}),
					),
				]
				.into(),
			),
			Collection(
				[Swap(
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				)]
				.into(),
			),
		));

		let errors = app
			.world_mut()
			.run_system_once(swap_equipped_items::<_Hands, _Forearms>);

		assert_eq!(
			vec![
				Err(handle_entity_error(SlotKey::BottomHand(Side::Left))),
				Err(handle_entity_error(SlotKey::BottomHand(Side::Right)))
			],
			errors
		)
	}

	#[test]
	fn log_handle_errors() {
		let mut app = App::new();
		let entity_without_handle = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			_Hands(
				[
					(SlotKey::BottomHand(Side::Left), entity_without_handle),
					(SlotKey::BottomHand(Side::Right), entity_without_handle),
				]
				.into(),
			),
			_Forearms(
				[
					(SlotKey::BottomHand(Side::Left), entity_without_handle),
					(SlotKey::BottomHand(Side::Right), entity_without_handle),
				]
				.into(),
			),
			Slots::<Skill>(
				[
					(
						SlotKey::BottomHand(Side::Left),
						Some(SkillItem {
							name: "left item",
							..default()
						}),
					),
					(
						SlotKey::BottomHand(Side::Right),
						Some(SkillItem {
							name: "right item",
							..default()
						}),
					),
				]
				.into(),
			),
			Collection(
				[Swap(
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				)]
				.into(),
			),
		));

		let errors = app
			.world_mut()
			.run_system_once(swap_equipped_items::<_Hands, _Forearms>);

		assert_eq!(
			vec![
				Err(handle_error(SlotKey::BottomHand(Side::Left))),
				Err(handle_error(SlotKey::BottomHand(Side::Right)))
			],
			errors
		)
	}
}
