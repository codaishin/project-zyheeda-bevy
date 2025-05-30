use crate::{
	Movement,
	PathOrWasd,
	systems::movement::{
		parse_directional_movement_key::DirectionalMovementInput,
		process_input::InputProcessComponent,
	},
	traits::change_per_frame::MinDistance,
};
use bevy::prelude::*;
use common::tools::speed::Speed;
use std::{marker::PhantomData, time::Duration};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct WasdInput<TMethod> {
	pub(crate) target: Vec3,
	_m: PhantomData<TMethod>,
}

impl<TMethod> From<Vec3> for WasdInput<TMethod> {
	fn from(target: Vec3) -> Self {
		Self {
			target,
			_m: PhantomData,
		}
	}
}

impl<TMethod> MinDistance for WasdInput<TMethod>
where
	TMethod: MinDistance,
{
	fn min_distance(speed: Speed, delta: Duration) -> f32 {
		TMethod::min_distance(speed, delta)
	}
}

impl<TMethod> InputProcessComponent for WasdInput<TMethod>
where
	TMethod: 'static,
{
	type TComponent = Movement<PathOrWasd<TMethod>>;
}

impl<TMethod> DirectionalMovementInput for WasdInput<TMethod> where TMethod: MinDistance {}
