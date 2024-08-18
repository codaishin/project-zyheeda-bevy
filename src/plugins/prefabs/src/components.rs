use bevy::prelude::{ChildBuilder, Component};
use std::sync::Arc;

#[derive(Component)]
pub struct WithChildren {
	pub(crate) spawn: Arc<dyn Fn(&mut ChildBuilder) + Sync + Send + 'static>,
}

impl WithChildren {
	pub fn delayed(spawn: impl Fn(&mut ChildBuilder) + Sync + Send + 'static) -> WithChildren {
		WithChildren {
			spawn: Arc::new(spawn),
		}
	}
}
