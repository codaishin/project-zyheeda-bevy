use crate::components::pass_layer::PassLayer;
use bevy::{
	camera::visibility::RenderLayers,
	core_pipeline::tonemapping::Tonemapping,
	post_process::bloom::Bloom,
	prelude::*,
	render::view::Hdr,
};
use common::traits::handles_graphics::StaticRenderLayers;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

const FIRST_PASS: usize = 0;
const SECOND_PASS: usize = 1;
const UI_PASS: usize = 2;

#[derive(Component, Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
#[require(Camera3d)]
pub struct WorldCamera;

#[derive(
	Component,
	SavableComponent,
	Debug,
	PartialEq,
	Eq,
	Hash,
	Default,
	Clone,
	Copy,
	Serialize,
	Deserialize,
)]
#[savable_component(id = "1st pass camera")]
#[require(
	WorldCamera,
	Tonemapping = Self,
	RenderLayers = Self,
	Hdr,
	Bloom
)]
pub struct FirstPass;

impl From<FirstPass> for Tonemapping {
	fn from(_: FirstPass) -> Self {
		Tonemapping::None
	}
}

impl From<FirstPass> for RenderLayers {
	fn from(_: FirstPass) -> Self {
		Self::layer(FIRST_PASS)
	}
}

impl From<FirstPass> for PassLayer {
	fn from(_: FirstPass) -> Self {
		PassLayer { layer: FIRST_PASS }
	}
}

#[derive(
	Component,
	SavableComponent,
	Debug,
	PartialEq,
	Eq,
	Hash,
	Default,
	Clone,
	Copy,
	Serialize,
	Deserialize,
)]
#[savable_component(id = "2nd pass camera")]
#[require(
	WorldCamera,
	Camera = Self,
	Tonemapping = Self,
	RenderLayers = Self,
	Hdr,
	Bloom,
)]
pub struct SecondPass;

impl From<SecondPass> for Camera {
	fn from(_: SecondPass) -> Self {
		Camera {
			order: SECOND_PASS as isize,
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
		RenderLayers::from_layers(&[FIRST_PASS, SECOND_PASS])
	}
}

impl From<SecondPass> for PassLayer {
	fn from(_: SecondPass) -> Self {
		PassLayer { layer: SECOND_PASS }
	}
}

#[derive(
	Component,
	SavableComponent,
	Debug,
	PartialEq,
	Eq,
	Hash,
	Default,
	Clone,
	Copy,
	Serialize,
	Deserialize,
)]
#[savable_component(id = "ui camera")]
#[require(
	WorldCamera,
	Camera = Self,
	Tonemapping = Self,
	RenderLayers = Self,
	Hdr,
)]
pub struct Ui;

impl From<Ui> for Camera {
	fn from(_: Ui) -> Self {
		Camera {
			order: UI_PASS as isize,
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
		RenderLayers::layer(UI_PASS)
	}
}

impl StaticRenderLayers for Ui {
	fn render_layers() -> RenderLayers {
		RenderLayers::from(Ui)
	}
}
