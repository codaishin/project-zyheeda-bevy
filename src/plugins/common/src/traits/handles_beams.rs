use crate::tools::Units;
use bevy::prelude::*;
use std::time::Duration;

pub trait HandlesBeams {
	fn beam_from<T>(value: &T) -> impl Bundle
	where
		T: BeamParameters;
}

pub trait BeamParameters {
	fn source(&self) -> Entity;
	fn target(&self) -> Entity;
	fn range(&self) -> Units;
	fn lifetime(&self) -> Duration;
}
