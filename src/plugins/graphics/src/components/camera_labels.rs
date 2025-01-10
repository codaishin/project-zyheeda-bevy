use super::insert_recursively::InsertRecursively;
use bevy::{
	core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
	prelude::*,
	render::{camera::RenderTarget, view::RenderLayers},
};
use common::traits::handles_graphics::StaticRenderLayers;

#[derive(Component, Debug, PartialEq, Default)]
#[require(Camera3d, Camera(Self::default), Tonemapping(Self::default))]
pub struct FirstPass;

impl From<FirstPass> for Camera {
	fn from(_: FirstPass) -> Self {
		Camera {
			hdr: true,
			..default()
		}
	}
}

impl From<FirstPass> for Tonemapping {
	fn from(_: FirstPass) -> Self {
		Tonemapping::None
	}
}

#[derive(Component, Debug, PartialEq)]
#[require(Camera3d, Tonemapping(FirstPass::default))]
pub struct FirstPassTexture {
	_private: (),
}

impl FirstPassTexture {
	fn new() -> Self {
		FirstPassTexture { _private: () }
	}

	pub(crate) fn from_image(handle: Handle<Image>) -> (FirstPassTexture, Camera) {
		let mut camera = Camera::from(FirstPass);
		camera.target = RenderTarget::Image(handle);

		(FirstPassTexture::new(), camera)
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[require(
	Camera3d,
	Camera(Self::default),
	Tonemapping(Self::default),
	Bloom,
	RenderLayers(Self::default)
)]
pub struct SecondPass;

impl SecondPass {
	const ORDER: usize = 1;
}

impl From<SecondPass> for Camera {
	fn from(_: SecondPass) -> Self {
		Camera {
			hdr: true,
			order: SecondPass::ORDER as isize,
			clear_color: ClearColorConfig::None,
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

impl From<SecondPass> for InsertRecursively<RenderLayers> {
	fn from(_: SecondPass) -> Self {
		const { InsertRecursively(RenderLayers::layer(SecondPass::ORDER)) }
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[require(
	Camera3d,
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
