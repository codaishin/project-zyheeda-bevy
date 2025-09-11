use crate::{
	Movement,
	PathOrWasd,
	systems::movement::insert_process_component::InputProcessComponent,
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct WasdInput<TMotion> {
	pub(crate) direction: Dir3,
	_m: PhantomData<TMotion>,
}

impl<TMotion> From<Dir3> for WasdInput<TMotion> {
	fn from(direction: Dir3) -> Self {
		Self {
			direction,
			_m: PhantomData,
		}
	}
}

impl<TMotion> InputProcessComponent for WasdInput<TMotion>
where
	TMotion: ThreadSafe,
{
	type TInputProcessComponent = Movement<PathOrWasd<TMotion>>;
}
