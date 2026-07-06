use crate::components::persistent_entity::PersistentEntity;
use bevy::ecs::system::SystemParam;

pub trait HandlesAgents {
	type TAgent: for<'w, 's> SystemParam<Item<'w, 's>: IntoIterator<Item = PersistentEntity>>;
}
