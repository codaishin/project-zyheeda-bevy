use crate::traits::{handles_loadout::LoadoutKey, loadout::ItemName};
use bevy::prelude::*;
use std::ops::DerefMut;

pub struct NotLoadedOut {
	pub entity: Entity,
}

impl From<NotLoadedOut> for Entity {
	fn from(NotLoadedOut { entity }: NotLoadedOut) -> Self {
		entity
	}
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
