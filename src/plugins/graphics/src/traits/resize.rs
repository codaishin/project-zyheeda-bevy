use bevy::{image::Image, render::render_resource::Extent3d};

pub(crate) trait Resize {
	fn resize(&mut self, size: Extent3d);
}

impl Resize for Image {
	fn resize(&mut self, size: Extent3d) {
		self.resize(size);
	}
}
