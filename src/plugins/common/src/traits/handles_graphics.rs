use bevy::{prelude::Component, render::view::RenderLayers};

pub trait UiRenderLayer {
	fn ui_render_layer() -> RenderLayers;
}

pub trait MainCamera {
	type TMainCamera: Component;
}
