use bevy::ecs::entity::Entity;
use bevy_rapier3d::geometry::CollidingEntities;

pub trait HasCollisions {
	fn collisions(&self) -> impl Iterator<Item = Entity> + '_;
}

impl HasCollisions for CollidingEntities {
	fn collisions(&self) -> impl Iterator<Item = Entity> + '_ {
		self.iter()
	}
}
