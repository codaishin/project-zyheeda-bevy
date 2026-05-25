use crate::traits::accessors::get::{ContextChanged, TryGetContext, TryGetContextMut, View};
use bevy::{ecs::component::Mutable, prelude::*};

impl<T> ContextChanged for Ref<'_, T>
where
	T: Component,
{
	fn context_changed(&self) -> bool {
		self.is_changed()
	}
}

impl<T, TKey> TryGetContext<TKey> for Query<'static, 'static, Ref<'static, T>>
where
	T: Component,
	TKey: View<Entity>,
{
	type TContext<'ctx> = Ref<'ctx, T>;

	fn try_get_context<'ctx>(
		query: &'ctx Query<Ref<T>>,
		key: TKey,
	) -> Option<Self::TContext<'ctx>> {
		query.get(key.view()).ok()
	}
}

impl<T, TKey> TryGetContextMut<TKey> for Query<'static, 'static, Mut<'static, T>>
where
	T: Component<Mutability = Mutable>,
	TKey: View<Entity>,
{
	type TContext<'ctx> = Mut<'ctx, T>;

	fn try_get_context_mut<'ctx>(
		query: &'ctx mut Query<Mut<T>>,
		key: TKey,
	) -> Option<Self::TContext<'ctx>> {
		query.get_mut(key.view()).ok()
	}
}

impl<T, TKey> TryGetContextMut<TKey> for Query<'static, 'static, &'static mut T>
where
	T: Component<Mutability = Mutable>,
	TKey: View<Entity>,
{
	type TContext<'ctx> = Mut<'ctx, T>;

	fn try_get_context_mut<'ctx>(
		query: &'ctx mut Query<&mut T>,

		key: TKey,
	) -> Option<Self::TContext<'ctx>> {
		query.get_mut(key.view()).ok()
	}
}
