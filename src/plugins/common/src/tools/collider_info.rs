use crate::components::outdated::Outdated;
use bevy::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ColliderInfo<T> {
	pub collider: T,
	pub root: Option<T>,
}

impl ColliderInfo<Entity> {
	pub fn with_component<TComponent: Component + Clone>(
		&self,
		get_component: impl Fn(Entity) -> Option<TComponent>,
	) -> Option<ColliderInfo<Outdated<TComponent>>> {
		Some(ColliderInfo {
			collider: Outdated {
				component: get_component(self.collider)?,
				entity: self.collider,
			},
			root: self.root.and_then(|root| {
				Some(Outdated {
					component: get_component(root)?,
					entity: root,
				})
			}),
		})
	}
}
