use bevy::prelude::*;
use std::{
	collections::{HashMap, HashSet},
	marker::PhantomData,
};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct OngoingInteractions<T> {
	pub(crate) interactions: HashMap<Entity, HashSet<Entity>>,
	_p: PhantomData<T>,
}

impl<T> Default for OngoingInteractions<T> {
	fn default() -> Self {
		Self {
			interactions: HashMap::default(),
			_p: PhantomData,
		}
	}
}

impl<T, TEntities> From<TEntities> for OngoingInteractions<T>
where
	TEntities: Into<HashMap<Entity, HashSet<Entity>>>,
{
	fn from(target: TEntities) -> Self {
		Self {
			interactions: target.into(),
			_p: PhantomData,
		}
	}
}
