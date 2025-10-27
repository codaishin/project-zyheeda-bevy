use crate::traits::accessors::get::{ContextChanged, EntityContext, EntityContextMut};
use bevy::{ecs::component::Mutable, prelude::*};

impl<T> ContextChanged for Ref<'_, T>
where
	T: Component,
{
	fn context_changed(&self) -> bool {
		self.is_changed()
	}
}

impl<T, TMarker> EntityContext<TMarker> for Query<'_, '_, Ref<'static, T>>
where
	T: Component,
{
	type TContext<'ctx> = Ref<'ctx, T>;

	fn get_entity_context<'ctx>(
		query: &'ctx Query<Ref<T>>,
		entity: Entity,
		_: TMarker,
	) -> Option<Self::TContext<'ctx>> {
		query.get(entity).ok()
	}
}

impl<T, TMarker> EntityContextMut<TMarker> for Query<'_, '_, Mut<'static, T>>
where
	T: Component<Mutability = Mutable>,
{
	type TContext<'ctx> = Mut<'ctx, T>;

	fn get_entity_context_mut<'ctx>(
		query: &'ctx mut Query<Mut<T>>,
		entity: Entity,
		_: TMarker,
	) -> Option<Self::TContext<'ctx>> {
		query.get_mut(entity).ok()
	}
}

impl<T, TMarker> EntityContextMut<TMarker> for Query<'_, '_, &'static mut T>
where
	T: Component<Mutability = Mutable>,
{
	type TContext<'ctx> = Mut<'ctx, T>;

	fn get_entity_context_mut<'ctx>(
		query: &'ctx mut Query<&mut T>,
		entity: Entity,
		_: TMarker,
	) -> Option<Self::TContext<'ctx>> {
		query.get_mut(entity).ok()
	}
}
