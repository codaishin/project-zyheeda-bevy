use bevy::ecs::query::QueryFilter;

pub trait ViewFilter {
	type TFilter: QueryFilter;
}
