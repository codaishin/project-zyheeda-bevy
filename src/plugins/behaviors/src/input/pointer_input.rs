use crate::{
	Movement,
	PathOrWasd,
	systems::movement::{
		insert_process_component::InputProcessComponent,
		parse_pointer_movement::PointMovementInput,
	},
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct PointerInput<TMethod> {
	pub(crate) target: Vec3,
	_m: PhantomData<TMethod>,
}

impl<TMethod> From<Vec3> for PointerInput<TMethod> {
	fn from(translation: Vec3) -> Self {
		Self {
			target: translation,
			_m: PhantomData,
		}
	}
}

impl<TMethod> InputProcessComponent for PointerInput<TMethod>
where
	TMethod: ThreadSafe,
{
	type TInputProcessComponent = Movement<PathOrWasd<TMethod>>;
}

impl<TMethod> PointMovementInput for PointerInput<TMethod> {}
