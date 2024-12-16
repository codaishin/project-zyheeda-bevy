use bevy::ecs::query::QueryFilter;

pub(crate) trait HasFilter {
	type TFilter: QueryFilter;
}
