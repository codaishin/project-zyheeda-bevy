use crate::traits::accessors::get::{ContextChanged, GetContext, GetContextMut};
use bevy::{
	ecs::{component::Mutable, system::SystemParamItem},
	prelude::*,
};
use std::ops::DerefMut;

impl<T> ContextChanged for &'_ Res<'_, T>
where
	T: Resource,
{
	fn context_changed(&self) -> bool {
		self.is_changed()
	}
}

impl<T, TKey> GetContext<TKey> for Res<'static, T>
where
	T: Resource,
{
	type TContext<'ctx> = &'ctx Res<'ctx, T>;

	fn get_context<'ctx>(param: &'ctx SystemParamItem<Self>, _: TKey) -> Self::TContext<'ctx> {
		param
	}
}

impl<T, TKey> GetContextMut<TKey> for ResMut<'static, T>
where
	T: Resource<Mutability = Mutable>,
{
	type TContext<'ctx> = &'ctx mut T;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		_: TKey,
	) -> Self::TContext<'ctx> {
		param.deref_mut()
	}
}
