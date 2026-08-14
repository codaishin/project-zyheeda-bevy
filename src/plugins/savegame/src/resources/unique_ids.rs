use bevy::prelude::*;
use common::prelude::*;
use std::{any::TypeId, collections::HashMap};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct UniqueIds(pub(crate) HashMap<UniqueComponentId, TypeId>);

impl<const N: usize> From<[(UniqueComponentId, TypeId); N]> for UniqueIds {
	fn from(value: [(UniqueComponentId, TypeId); N]) -> Self {
		Self(HashMap::from(value))
	}
}
