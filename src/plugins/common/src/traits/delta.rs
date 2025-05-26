use super::thread_safe::ThreadSafe;
use bevy::prelude::*;
use std::time::Duration;

pub trait Delta: internal::Delta {
	fn delta(time: Res<Time<Self::TTime>>) -> Duration {
		time.delta()
	}
}

impl<T> Delta for T where T: internal::Delta {}

mod internal {
	use super::*;

	pub trait Delta {
		type TTime: Default + ThreadSafe;
	}

	impl Delta for Update {
		type TTime = Virtual;
	}

	impl Delta for FixedUpdate {
		type TTime = Fixed;
	}

	impl Delta for FixedPostUpdate {
		type TTime = Fixed;
	}
}
