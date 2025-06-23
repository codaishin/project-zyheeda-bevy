use bevy::{
	core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
	prelude::*,
	render::view::RenderLayers,
};
use common::traits::handles_graphics::StaticRenderLayers;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
#[require(Camera3d)]
pub struct WorldCamera;

#[derive(Component, Debug, PartialEq, Eq, Hash, Default, Clone, Copy, Serialize, Deserialize)]
#[require(WorldCamera,Tonemapping = Self, Bloom)]
pub struct FirstPass;

impl From<FirstPass> for Tonemapping {
	fn from(_: FirstPass) -> Self {
		Tonemapping::None
	}
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Default, Clone, Copy, Serialize, Deserialize)]
#[require(
	WorldCamera,
	Camera = Self,
	Tonemapping = Self,
	Bloom,
	RenderLayers = Self::with_default_layer(),
)]
pub struct SecondPass;

impl SecondPass {
	const ORDER: usize = 1;

	fn with_default_layer() -> RenderLayers {
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
		const { Tonemapping::TonyMcMapface }
	}
}

impl From<SecondPass> for RenderLayers {
	fn from(_: SecondPass) -> Self {
		const { RenderLayers::layer(SecondPass::ORDER) }
	}
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Default, Clone, Copy, Serialize, Deserialize)]
#[require(
	WorldCamera,
	Camera = Self,
	Tonemapping = Self,
	RenderLayers = Self,
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
