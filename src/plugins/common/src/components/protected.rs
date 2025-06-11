use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct Protected<TComponent>(PhantomData<TComponent>)
where
	TComponent: Component;

impl<TComponent> Default for Protected<TComponent>
where
	TComponent: Component,
{
	fn default() -> Self {
		Self(PhantomData)
	}
}
