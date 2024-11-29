use bevy::prelude::*;
use std::time::Duration;

pub trait HandlesLifetime {
	fn lifetime(duration: Duration) -> impl Bundle;
}
