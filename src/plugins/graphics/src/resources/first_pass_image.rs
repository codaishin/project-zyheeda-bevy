use bevy::{
	asset::RenderAssetUsages,
	prelude::*,
	render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct FirstPassImage {
	pub(crate) handle: Handle<Image>,
}

impl FirstPassImage {
	pub(crate) fn instantiate(mut images: ResMut<Assets<Image>>) -> Self {
		let mut image = Image::new_fill(
			Extent3d::default(),
			TextureDimension::D2,
			&[0, 0, 0, 255],
			TextureFormat::Bgra8UnormSrgb,
			RenderAssetUsages::default(),
		);
		image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
			| TextureUsages::RENDER_ATTACHMENT
			| TextureUsages::COPY_SRC;

		Self {
			handle: images.add(image),
		}
	}
}
