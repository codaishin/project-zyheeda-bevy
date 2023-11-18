use crate::errors::Error;
use bevy::ecs::system::In;
use tracing::error;

pub fn log(result: In<Result<(), Error>>) {
	let Err(error) = result.0 else {
		return;
	};

	error!("{:?}", error)
}
