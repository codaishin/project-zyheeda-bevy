use bevy::{
	asset::{Asset, Handle, LoadedFolder},
	prelude::Resource,
};
use std::fmt::{Debug, Formatter, Result};

#[derive(Resource)]
pub(crate) struct SkillFolder<TFolder: Asset = LoadedFolder>(pub Handle<TFolder>);

impl<TFolder: Asset> Default for SkillFolder<TFolder> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<TFolder: Asset> Debug for SkillFolder<TFolder> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		f.debug_tuple("SkillFolder").field(&self.0).finish()
	}
}

impl<TFolder: Asset> PartialEq for SkillFolder<TFolder> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}
