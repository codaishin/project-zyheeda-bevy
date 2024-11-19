use bevy::prelude::{ChildBuilder, Component};
use common::traits::prefab::AfterInstantiation;
use std::sync::Arc;

#[derive(Component, Clone)]
pub struct SpawnAfterInstantiation {
	pub(crate) spawn: Arc<dyn Fn(&mut ChildBuilder) + Sync + Send + 'static>,
}

impl AfterInstantiation for SpawnAfterInstantiation {
	fn spawn(
		spawn_fn: impl Fn(&mut ChildBuilder) + Sync + Send + 'static,
	) -> impl bevy::prelude::Bundle {
		SpawnAfterInstantiation {
			spawn: Arc::new(spawn_fn),
		}
	}
}
