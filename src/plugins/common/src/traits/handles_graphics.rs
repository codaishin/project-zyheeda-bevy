use bevy::{camera::visibility::RenderLayers, prelude::Component};

pub trait StaticRenderLayers {
	fn render_layers() -> RenderLayers;
}

pub trait UiCamera {
	type TUiCamera: Component + StaticRenderLayers;
}

pub trait FirstPassCamera {
	type TFirstPassCamera: Component;
}

pub trait WorldCameras {
	type TWorldCameras: Component;
}
