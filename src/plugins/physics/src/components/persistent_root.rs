use bevy::prelude::*;
use common::prelude::*;
use std::ops::Deref;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct PersistentRoot(pub(crate) PersistentEntity);

impl Deref for PersistentRoot {
	type Target = PersistentEntity;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
