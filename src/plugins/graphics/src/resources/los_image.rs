use bevy::prelude::*;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct LoSImageAtlas {
	pub(crate) handle: Handle<Image>,
}

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct LoSImageCubemap {
	pub(crate) handle: Handle<Image>,
}
