use crate::{
	Movement,
	PathOrWasd,
	systems::movement::insert_process_component::InputProcessComponent,
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct WasdInput<TMethod> {
	pub(crate) direction: Dir3,
	_m: PhantomData<TMethod>,
}

impl<TMethod> From<Dir3> for WasdInput<TMethod> {
	fn from(direction: Dir3) -> Self {
		Self {
			direction,
			_m: PhantomData,
		}
	}
}

impl<TMethod> InputProcessComponent for WasdInput<TMethod>
where
	TMethod: ThreadSafe,
{
	type TInputProcessComponent = Movement<PathOrWasd<TMethod>>;
}
