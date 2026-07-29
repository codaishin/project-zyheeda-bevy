use bevy::{prelude::*, render::extract_resource::ExtractResource};

#[derive(Resource, Debug, PartialEq, Clone)]
pub(crate) struct LoSImageShared {
	pub(crate) handle: Handle<Image>,
}

#[derive(Resource, ExtractResource, Debug, PartialEq, Clone)]
pub(crate) struct LoSImageCubemap {
	pub(crate) handle: Handle<Image>,
}
