use bevy::{ecs::query::QueryFilter, prelude::Bundle};

pub trait ItemView<TKey> {
	type TFilter: QueryFilter + 'static;
	type TViewComponents: Bundle + Default + Clone + Sync + Send + 'static;

	fn view_entity_name(key: &TKey) -> &'static str;
}
