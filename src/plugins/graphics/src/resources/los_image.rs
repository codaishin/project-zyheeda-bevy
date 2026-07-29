use bevy::{prelude::*, render::extract_resource::ExtractResource};

#[derive(Resource, ExtractResource, Debug, PartialEq, Clone)]
pub(crate) struct LoSImageAtlas {
	pub(crate) handle: Handle<Image>,
}

#[derive(Resource, ExtractResource, Debug, PartialEq, Clone)]
pub(crate) struct LoSImageCubemap {
	pub(crate) handle: Handle<Image>,
}
