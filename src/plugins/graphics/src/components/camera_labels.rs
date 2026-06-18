use crate::{PostProcessCamera, components::model_render_layers::ModelRenderLayers};
use bevy::{
	camera::visibility::{Layer, RenderLayers},
	color::palettes::tailwind,
	core_pipeline::{prepass::DepthPrepass, tonemapping::Tonemapping},
	light::light_consts::lux,
	post_process::bloom::Bloom,
	prelude::*,
	render::{extract_component::ExtractComponent, view::Hdr},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

const WORLD_PASS: Layer = 0;
const AGENTS_PASS: Layer = 1;
const VISIBILITY_PASS: Layer = 2;
const OUTLINE_PASS: Layer = 3;
const COMPOSITE_PASS: Layer = 4;
const UI_PASS: Layer = 5;

#[derive(Component, Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub struct MoveWithPlayerCam;

#[derive(
	Component,
	ExtractComponent,
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
#[savable_component(id = "world pass camera")]
#[require(
	Camera3d,
	MoveWithPlayerCam,
	Camera::from(Self),
	RenderLayers::from(Self),
	Tonemapping::from(Self),
	Hdr,
	Bloom,
	Msaa::Off,
	DepthPrepass
)]
pub struct WorldPass;

impl From<WorldPass> for Camera {
	fn from(_: WorldPass) -> Self {
		Camera {
			order: WORLD_PASS as isize,
			..default()
		}
	}
}

impl From<WorldPass> for RenderLayers {
	fn from(_: WorldPass) -> Self {
		Self::layer(WORLD_PASS)
	}
}

impl From<WorldPass> for Tonemapping {
	fn from(_: WorldPass) -> Self {
		Tonemapping::None
	}
}

impl From<WorldPass> for ModelRenderLayers {
	fn from(_: WorldPass) -> Self {
		const WORLD_LAYERS: &[Layer] = &[WORLD_PASS, VISIBILITY_PASS];
		ModelRenderLayers::from(WORLD_LAYERS)
	}
}

#[derive(
	Component,
	ExtractComponent,
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
#[savable_component(id = "outline pass camera")]
#[require(
	Camera3d,
	MoveWithPlayerCam,
	Camera::from(Self),
	RenderLayers::from(Self),
	Tonemapping::from(Self),
	DepthPrepass,
	Msaa::Off
)]
pub(crate) struct OutlinePass;

impl From<OutlinePass> for Camera {
	fn from(_: OutlinePass) -> Self {
		Camera {
			order: OUTLINE_PASS as isize,
			// Clear color needs to have an alpha of `0.0`, because the outline shading tests against
			// the alpha. If we want the full color on the outline pass result, we also need some light
			// on the outline render layer.
			clear_color: Color::NONE.into(),
			..default()
		}
	}
}

impl From<OutlinePass> for RenderLayers {
	fn from(_: OutlinePass) -> Self {
		RenderLayers::layer(OUTLINE_PASS)
	}
}

impl From<OutlinePass> for ModelRenderLayers {
	fn from(_: OutlinePass) -> Self {
		ModelRenderLayers::from(OUTLINE_PASS)
	}
}

impl From<OutlinePass> for Tonemapping {
	fn from(_: OutlinePass) -> Self {
		Tonemapping::None
	}
}

#[derive(
	Component,
	ExtractComponent,
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
#[savable_component(id = "agents pass camera")]
#[require(
	Camera3d,
	MoveWithPlayerCam,
	Camera::from(Self),
	RenderLayers::from(Self),
	Tonemapping::from(Self),
	DepthPrepass,
	Msaa::Off
)]
pub(crate) struct AgentsPass;

impl From<AgentsPass> for Camera {
	fn from(_: AgentsPass) -> Self {
		Camera {
			order: AGENTS_PASS as isize,
			clear_color: Color::NONE.into(),
			..default()
		}
	}
}

impl From<AgentsPass> for RenderLayers {
	fn from(_: AgentsPass) -> Self {
		RenderLayers::layer(AGENTS_PASS)
	}
}

impl From<AgentsPass> for ModelRenderLayers {
	fn from(_: AgentsPass) -> Self {
		ModelRenderLayers::from(AGENTS_PASS)
	}
}

impl From<AgentsPass> for Tonemapping {
	fn from(_: AgentsPass) -> Self {
		Tonemapping::None
	}
}

#[derive(
	Component,
	ExtractComponent,
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
#[savable_component(id = "visibility pass camera")]
#[require(
	Camera3d,
	MoveWithPlayerCam,
	Camera::from(Self),
	RenderLayers::from(Self),
	Tonemapping::from(Self),
	Hdr
)]
pub(crate) struct VisibilityPass;

impl From<VisibilityPass> for Camera {
	fn from(_: VisibilityPass) -> Self {
		Camera {
			order: VISIBILITY_PASS as isize,
			clear_color: Color::NONE.into(),
			..default()
		}
	}
}

impl From<VisibilityPass> for RenderLayers {
	fn from(_: VisibilityPass) -> Self {
		RenderLayers::layer(VISIBILITY_PASS)
	}
}

impl From<VisibilityPass> for ModelRenderLayers {
	fn from(_: VisibilityPass) -> Self {
		ModelRenderLayers::from(VISIBILITY_PASS)
	}
}

impl From<VisibilityPass> for Tonemapping {
	fn from(_: VisibilityPass) -> Self {
		Tonemapping::None
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
#[savable_component(id = "composite pass camera")]
#[require(
	Camera3d,
	MoveWithPlayerCam,
	PostProcessCamera::from(Self),
	Camera::from(Self),
	RenderLayers::from(Self),
	Tonemapping::from(Self),
	Hdr,
	Bloom
)]
pub(crate) struct CompositePass;

impl From<CompositePass> for Camera {
	fn from(_: CompositePass) -> Self {
		Camera {
			order: COMPOSITE_PASS as isize,
			..default()
		}
	}
}

impl From<CompositePass> for Tonemapping {
	fn from(_: CompositePass) -> Self {
		Tonemapping::TonyMcMapface
	}
}

impl From<CompositePass> for RenderLayers {
	fn from(_: CompositePass) -> Self {
		RenderLayers::from_layers(&[WORLD_PASS, AGENTS_PASS, COMPOSITE_PASS])
	}
}

impl From<CompositePass> for ModelRenderLayers {
	fn from(_: CompositePass) -> Self {
		ModelRenderLayers::from(COMPOSITE_PASS)
	}
}

impl From<CompositePass> for PostProcessCamera {
	fn from(_: CompositePass) -> Self {
		PostProcessCamera {
			outline_color: (tailwind::GREEN_600 * 2.).into(),
			see_through_color: tailwind::GRAY_50.into(),
		}
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
#[savable_component(id = "world light")]
#[require(
	MoveWithPlayerCam,
	RenderLayers::from(Self),
	DirectionalLight::from(Self)
)]
pub(crate) struct WorldLight;

impl From<WorldLight> for RenderLayers {
	fn from(_: WorldLight) -> Self {
		RenderLayers::from_layers(&[WORLD_PASS, AGENTS_PASS])
	}
}

impl From<WorldLight> for DirectionalLight {
	fn from(_: WorldLight) -> Self {
		DirectionalLight {
			illuminance: lux::AMBIENT_DAYLIGHT,
			..default()
		}
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
#[savable_component(id = "ui pass camera")]
#[require(
	Camera3d,
	MoveWithPlayerCam,
	Camera::from(Self),
	RenderLayers::from(Self),
	Tonemapping::from(Self),
	Hdr
)]
pub struct UiPass;

impl From<UiPass> for Camera {
	fn from(_: UiPass) -> Self {
		Camera {
			order: UI_PASS as isize,
			clear_color: ClearColorConfig::None,
			..default()
		}
	}
}

impl From<UiPass> for RenderLayers {
	fn from(_: UiPass) -> Self {
		RenderLayers::layer(UI_PASS)
	}
}

impl From<UiPass> for Tonemapping {
	fn from(_: UiPass) -> Self {
		Tonemapping::None
	}
}
