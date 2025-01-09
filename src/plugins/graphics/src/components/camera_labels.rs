use bevy::{
	core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
	prelude::*,
	render::{camera::RenderTarget, view::RenderLayers},
};

#[derive(Component, Debug, PartialEq)]
#[require(Camera3d, Camera(Self::camera), Tonemapping(Self::tonemapping))]
pub struct FirstPass;

impl FirstPass {
	fn camera() -> Camera {
		Camera {
			hdr: true,
			..default()
		}
	}

	fn tonemapping() -> Tonemapping {
		Tonemapping::None
	}
}

#[derive(Component, Debug, PartialEq)]
#[require(Camera3d, Tonemapping(FirstPass::tonemapping))]
pub struct FirstPassTexture {
	_private: (),
}

impl FirstPassTexture {
	fn new() -> Self {
		FirstPassTexture { _private: () }
	}

	pub(crate) fn from_image(handle: Handle<Image>) -> (FirstPassTexture, Camera) {
		let mut camera = FirstPass::camera();
		camera.target = RenderTarget::Image(handle);

		(FirstPassTexture::new(), camera)
	}
}

#[derive(Component, Debug, PartialEq)]
#[require(
	Camera3d,
	Camera(SecondPass::camera),
	Tonemapping(Self::tonemapping),
	Bloom,
	RenderLayers(Self::render_layers)
)]
pub struct SecondPass;

impl SecondPass {
	const ORDER: usize = 1;

	pub(crate) fn render_layers() -> RenderLayers {
		const { RenderLayers::layer(Self::ORDER) }
	}

	fn camera() -> Camera {
		Camera {
			hdr: true,
			order: Self::ORDER as isize,
			clear_color: ClearColorConfig::None,
			..default()
		}
	}

	fn tonemapping() -> Tonemapping {
		Tonemapping::TonyMcMapface
	}
}

#[derive(Component, Debug, PartialEq)]
#[require(
	Camera3d,
	Camera(Ui::camera),
	Tonemapping(Self::tonemapping),
	RenderLayers(Self::render_layers)
)]
pub struct Ui;

impl Ui {
	const ORDER: usize = 2;

	pub(crate) fn render_layers() -> RenderLayers {
		const { RenderLayers::layer(Self::ORDER) }
	}

	fn camera() -> Camera {
		Camera {
			order: Self::ORDER as isize,
			hdr: true,
			clear_color: ClearColorConfig::None,
			..default()
		}
	}

	fn tonemapping() -> Tonemapping {
		Tonemapping::None
	}
}
