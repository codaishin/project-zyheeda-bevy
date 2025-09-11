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
pub(crate) struct PointerInput<TMotion> {
	pub(crate) target: Vec3,
	_m: PhantomData<TMotion>,
}

impl<TMotion> From<Vec3> for PointerInput<TMotion> {
	fn from(translation: Vec3) -> Self {
		Self {
			target: translation,
			_m: PhantomData,
		}
	}
}

impl<TMotion> InputProcessComponent for PointerInput<TMotion>
where
	TMotion: ThreadSafe,
{
	type TInputProcessComponent = Movement<PathOrWasd<TMotion>>;
}

impl<TMotion> PointMovementInput for PointerInput<TMotion> {}
