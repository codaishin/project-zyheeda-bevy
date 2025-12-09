use bevy::ecs::query::QueryFilter;

pub(crate) trait QueryFilterDefinition {
	type TFilter: QueryFilter;
}
