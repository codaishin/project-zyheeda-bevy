use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

#[derive(Component)]
pub(crate) struct WithNode(pub(crate) Box<dyn Fn(&mut Node) + Sync + Send + 'static>);

impl WithNode {
	pub(crate) fn modify_or_insert<T>(mod_func: T) -> Self
	where
		T: Fn(&mut Node) + ThreadSafe,
	{
		Self(Box::new(mod_func))
	}
}
