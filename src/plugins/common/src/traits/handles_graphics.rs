use bevy::{prelude::Component, render::view::RenderLayers};

pub trait StaticRenderLayers {
	fn render_layers() -> RenderLayers;
}

pub trait UiCamera {
	type TUiCamera: Component + StaticRenderLayers;
}

pub trait PlayerCameras {
	type TPlayerCamera: Component;
}
