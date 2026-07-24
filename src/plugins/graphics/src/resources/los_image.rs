use bevy::prelude::*;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct LoSImage {
	pub(crate) handle: Handle<Image>,
}
