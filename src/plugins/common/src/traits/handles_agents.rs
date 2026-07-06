use crate::components::persistent_entity::PersistentEntity;
use bevy::ecs::{query::QueryFilter, system::SystemParam};

pub trait HandlesAgents {
	type TAgent<TFilter>: for<'w, 's> SystemParam<
		Item<'w, 's>: IntoIterator<Item = PersistentEntity>,
	>
	where
		TFilter: QueryFilter + 'static;
}
