use crate::components::{
	combos::Combos,
	combos_time_out::CombosTimeOut,
	inventory::Inventory,
	queue::Queue,
	skill_executer::SkillExecuter,
	slots::Slots,
	swapper::Swapper,
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		load_asset::LoadAsset,
		loadout::LoadoutConfig,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::{marker::PhantomData, time::Duration};

#[derive(Component, Debug)]
#[require(
	Combos,
	CombosTimeOut = CombosTimeOut::after(Duration::from_secs(2)),
	Queue,
	SkillExecuter,
	Swapper,
)]
pub(crate) struct Loadout<T>(PhantomData<T>)
where
	T: LoadoutConfig + ThreadSafe;

impl<T> Loadout<T>
where
	T: LoadoutConfig + ThreadSafe,
{
	pub(crate) fn insert(
		trigger: Trigger<OnInsert, T>,
		agents: Query<&T>,
		mut commands: ZyheedaCommands,
		mut assets: ResMut<AssetServer>,
	) {
		let entity = trigger.target();
		let Ok(agent) = agents.get(entity) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert_if_new((
				Self::default(),
				Inventory::from(
					agent
						.inventory()
						.into_iter()
						.map(|path| path.map(|path| assets.load_asset(path))),
				),
				Slots::from(
					agent
						.slots()
						.into_iter()
						.map(|(key, path)| (key, path.map(|path| assets.load_asset(path)))),
				),
			));
		});
	}
}

impl<T> Default for Loadout<T>
where
	T: LoadoutConfig + ThreadSafe,
{
	fn default() -> Self {
		Self(PhantomData)
	}
}
