use bevy::prelude::Component;
use std::time::Duration;

pub trait HandlesLifetime {
	type TLifetime: From<Duration> + Component;
}
