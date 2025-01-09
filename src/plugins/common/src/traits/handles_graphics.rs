use bevy::render::view::RenderLayers;

pub trait UiRenderLayer {
	fn ui_render_layer() -> RenderLayers;
}
