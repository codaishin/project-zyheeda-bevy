use crate::traits::{handles_loadout::LoadoutKey, loadout::ItemName};
use bevy::prelude::*;
use macros::EntityKey;
use std::ops::DerefMut;

#[derive(EntityKey)]
pub struct NotLoadedOut {
	pub entity: Entity,
}

pub trait InsertDefaultLoadout {
	fn insert_default_loadout<TItems>(&mut self, loadout: TItems)
	where
		TItems: IntoIterator<Item = (LoadoutKey, Option<ItemName>)>;
}

impl<T> InsertDefaultLoadout for T
where
	T: DerefMut<Target: InsertDefaultLoadout>,
{
	fn insert_default_loadout<TItems>(&mut self, loadout: TItems)
	where
		TItems: IntoIterator<Item = (LoadoutKey, Option<ItemName>)>,
	{
		self.deref_mut().insert_default_loadout(loadout);
	}
}
