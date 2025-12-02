use crate::traits::{handles_loadout::LoadoutKey, loadout::ItemName};
use bevy::prelude::*;

pub struct NotLoadedOut {
	pub entity: Entity,
}

pub trait InsertDefaultLoadout {
	fn insert_default_loadout<TItems>(&mut self, loadout: TItems)
	where
		TItems: IntoIterator<Item = (LoadoutKey, ItemName)>;
}
