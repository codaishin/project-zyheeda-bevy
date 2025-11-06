use crate::traits::accessors::get::{ContextChanged, GetContext, GetContextMut};
use bevy::{ecs::component::Mutable, prelude::*};

impl<T> ContextChanged for Ref<'_, T>
where
	T: Component,
{
	fn context_changed(&self) -> bool {
		self.is_changed()
	}
}

impl<T, TKey> GetContext<TKey> for Query<'_, '_, Ref<'static, T>>
where
	T: Component,
	TKey: Into<Entity>,
{
	type TContext<'ctx> = Ref<'ctx, T>;

	fn get_context<'ctx>(query: &'ctx Query<Ref<T>>, key: TKey) -> Option<Self::TContext<'ctx>> {
		query.get(key.into()).ok()
	}
}

impl<T, TKey> GetContextMut<TKey> for Query<'_, '_, Mut<'static, T>>
where
	T: Component<Mutability = Mutable>,
	TKey: Into<Entity>,
{
	type TContext<'ctx> = Mut<'ctx, T>;

	fn get_context_mut<'ctx>(
		query: &'ctx mut Query<Mut<T>>,
		key: TKey,
	) -> Option<Self::TContext<'ctx>> {
		query.get_mut(key.into()).ok()
	}
}

impl<T, TKey> GetContextMut<TKey> for Query<'_, '_, &'static mut T>
where
	T: Component<Mutability = Mutable>,
	TKey: Into<Entity>,
{
	type TContext<'ctx> = Mut<'ctx, T>;

	fn get_context_mut<'ctx>(
		query: &'ctx mut Query<&mut T>,

		key: TKey,
	) -> Option<Self::TContext<'ctx>> {
		query.get_mut(key.into()).ok()
	}
}
