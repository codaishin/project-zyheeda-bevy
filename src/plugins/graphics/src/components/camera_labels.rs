use bevy::{
	core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
	prelude::*,
	render::{camera::RenderTarget, view::RenderLayers},
};
use common::traits::handles_graphics::StaticRenderLayers;

#[derive(Component, Debug, PartialEq, Default)]
#[require(Camera3d)]
pub struct PlayerCamera;

#[derive(Component, Debug, PartialEq)]
#[require(PlayerCamera, Tonemapping(Self::new), Bloom)]
pub struct FirstPass {
	_private: (),
}

impl FirstPass {
	fn new() -> Self {
		FirstPass { _private: () }
	}

	pub(crate) fn with_target_image(handle: Handle<Image>) -> (FirstPass, Camera) {
		(
			FirstPass::new(),
			Camera {
				hdr: true,
				target: RenderTarget::Image(handle),
				..default()
			},
		)
	}
}

impl From<FirstPass> for Tonemapping {
	fn from(_: FirstPass) -> Self {
		Tonemapping::None
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[require(
	PlayerCamera,
	Camera(Self::default),
	Tonemapping(Self::default),
	Bloom,
	RenderLayers(Self::camera_render_layers)
)]
pub struct SecondPass;

impl SecondPass {
	const ORDER: usize = 1;

	fn camera_render_layers() -> RenderLayers {
		RenderLayers::from_layers(&[0, Self::ORDER])
	}
}

impl From<SecondPass> for Camera {
	fn from(_: SecondPass) -> Self {
		Camera {
			hdr: true,
			order: SecondPass::ORDER as isize,
			..default()
		}
	}
}

impl From<SecondPass> for Tonemapping {
	fn from(_: SecondPass) -> Self {
		Tonemapping::TonyMcMapface
	}
}

impl From<SecondPass> for RenderLayers {
	fn from(_: SecondPass) -> Self {
		const { RenderLayers::layer(SecondPass::ORDER) }
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[require(
	PlayerCamera,
	Camera(Ui::default),
	Tonemapping(Self::default),
	RenderLayers(Self::default)
)]
pub struct Ui;

impl Ui {
	const ORDER: usize = 2;
}

impl From<Ui> for Camera {
	fn from(_: Ui) -> Self {
		Camera {
			order: Ui::ORDER as isize,
			hdr: true,
			clear_color: ClearColorConfig::None,
			..default()
		}
	}
}

impl From<Ui> for Tonemapping {
	fn from(_: Ui) -> Self {
		Tonemapping::None
	}
}

impl From<Ui> for RenderLayers {
	fn from(_: Ui) -> Self {
		const { RenderLayers::layer(Ui::ORDER) }
	}
}

impl StaticRenderLayers for Ui {
	fn render_layers() -> RenderLayers {
		RenderLayers::from(Ui)
	}
}
