use crate::{
	PostProcessCamera,
	components::{
		model_render_layers::ModelRenderLayers,
		only_depth_prepass::OnlyDepthPrepass,
		post_process_camera::PostProcessArgs,
	},
};
use bevy::{
	camera::{
		CameraOutputMode,
		ComputedCameraValues,
		visibility::{Layer, RenderLayers},
	},
	color::palettes::tailwind,
	core_pipeline::{prepass::DepthPrepass, tonemapping::Tonemapping},
	ecs::system::StaticSystemParam,
	post_process::bloom::Bloom,
	prelude::*,
	render::{extract_component::ExtractComponent, view::Hdr},
};
use common::{
	errors::Unreachable,
	tools::pixel::Pixel,
	traits::prefab::{Prefab, PrefabEntityCommands},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

const WORLD_PASS: Layer = 0;
const AGENTS_PASS: Layer = 1;
const VISIBILITY_PASS: Layer = 2;
const EFFECT_LIGHT_PASS: Layer = 3;
const OUTLINE_PASS: Layer = 4;
const COMPOSITE_PASS: Layer = 5;
const UI_PASS: Layer = 6;

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
#[cfg_attr(debug_assertions, require(Name::from("World Camera")))]
pub struct WorldPass;

impl From<WorldPass> for Camera {
	fn from(_: WorldPass) -> Self {
		new_camera(CameraConfig {
			order: WORLD_PASS,
			..default()
		})
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
	OnlyDepthPrepass,
	Msaa::Off
)]
#[cfg_attr(debug_assertions, require(Name::from("Outline Camera")))]
pub(crate) struct OutlinePass;

impl From<OutlinePass> for Camera {
	fn from(_: OutlinePass) -> Self {
		new_camera(CameraConfig {
			order: OUTLINE_PASS,
			// Clear color needs to have an alpha of `0.0`, because the outline shading tests against
			// the alpha. If we want the full color on the outline pass result, we also need some light
			// on the outline render layer.
			clear_color: Color::NONE.into(),
		})
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
#[cfg_attr(debug_assertions, require(Name::from("Agents Camera")))]
pub(crate) struct AgentsPass;

impl From<AgentsPass> for Camera {
	fn from(_: AgentsPass) -> Self {
		new_camera(CameraConfig {
			order: AGENTS_PASS,
			clear_color: Color::NONE.into(),
		})
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
#[cfg_attr(debug_assertions, require(Name::from("Visibility Camera")))]
pub(crate) struct VisibilityPass;

impl From<VisibilityPass> for Camera {
	fn from(_: VisibilityPass) -> Self {
		new_camera(CameraConfig {
			order: VISIBILITY_PASS,
			clear_color: Color::NONE.into(),
		})
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
#[savable_component(id = "effect light camera")]
#[require(
	Camera3d,
	MoveWithPlayerCam,
	Camera::from(Self),
	RenderLayers::from(Self),
	Tonemapping::from(Self),
	Bloom,
	Hdr
)]
#[cfg_attr(debug_assertions, require(Name::from("Effect Light Camera")))]
pub(crate) struct EffectLightPass;

impl From<EffectLightPass> for Camera {
	fn from(_: EffectLightPass) -> Self {
		new_camera(CameraConfig {
			order: EFFECT_LIGHT_PASS,
			clear_color: Color::NONE.into(),
		})
	}
}

impl From<EffectLightPass> for RenderLayers {
	fn from(_: EffectLightPass) -> Self {
		RenderLayers::layer(EFFECT_LIGHT_PASS)
	}
}

impl From<EffectLightPass> for ModelRenderLayers {
	fn from(_: EffectLightPass) -> Self {
		ModelRenderLayers::from(EFFECT_LIGHT_PASS)
	}
}

impl From<EffectLightPass> for Tonemapping {
	fn from(_: EffectLightPass) -> Self {
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
#[cfg_attr(debug_assertions, require(Name::from("Composite Camera")))]
pub(crate) struct CompositePass;

impl From<CompositePass> for Camera {
	fn from(_: CompositePass) -> Self {
		new_camera(CameraConfig {
			order: COMPOSITE_PASS,
			..default()
		})
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
		const COMPOSITE_LAYERS: &[Layer] = &[EFFECT_LIGHT_PASS, COMPOSITE_PASS];
		ModelRenderLayers::from(COMPOSITE_LAYERS)
	}
}

impl From<CompositePass> for PostProcessCamera {
	fn from(_: CompositePass) -> Self {
		PostProcessCamera::new(PostProcessArgs {
			outline_color: tailwind::GREEN_600 * 2.,
			see_through_color: tailwind::GRAY_50,
			outline_width: Pixel(1.5),
			dark_region_light_factor: 0.01,
		})
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
#[component(immutable)]
#[require(MoveWithPlayerCam, RenderLayers::from(Self), Visibility, Transform)]
#[cfg_attr(debug_assertions, require(Name::from("World Light")))]
pub(crate) struct WorldLight;

impl From<WorldLight> for RenderLayers {
	fn from(_: WorldLight) -> Self {
		RenderLayers::from_layers(&[WORLD_PASS, AGENTS_PASS])
	}
}

impl Prefab<()> for WorldLight {
	type TError = Unreachable;
	type TSystemParam = ();

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<()>,
	) -> Result<(), Self::TError> {
		let illuminance = 2500.;
		let to_left = Quat::from_axis_angle(Vec3::Y, (-25_f32).to_radians());
		let to_right = Quat::from_axis_angle(Vec3::Y, (25_f32).to_radians());

		entity
			.with_child((
				Transform::default().with_rotation(to_left),
				DirectionalLight {
					illuminance,
					..default()
				},
			))
			.with_child((
				Transform::default().with_rotation(to_right),
				DirectionalLight {
					illuminance,
					..default()
				},
			));

		Ok(())
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
#[cfg_attr(debug_assertions, require(Name::from("UI Camera")))]
pub(crate) struct UiPass;

impl UiPass {
	pub(crate) const DEFAULT_TRANSFORM: &GlobalTransform = &GlobalTransform::IDENTITY;
	pub(crate) const DEFAULT_CAMERA: &Camera = &new_camera(CameraConfig {
		order: UI_PASS,
		clear_color: ClearColorConfig::None,
	});
}

impl From<UiPass> for Camera {
	fn from(_: UiPass) -> Self {
		UiPass::DEFAULT_CAMERA.clone()
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

#[derive(Default)]
struct CameraConfig {
	order: usize,
	clear_color: ClearColorConfig,
}

const fn new_camera(CameraConfig { order, clear_color }: CameraConfig) -> Camera {
	Camera {
		order: order as isize,
		clear_color,
		msaa_writeback: MsaaWriteback::Auto,
		invert_culling: false,
		is_active: true,
		viewport: None,
		sub_camera_view: None,
		computed: ComputedCameraValues {
			clip_from_view: Mat4::IDENTITY,
			target_info: None,
			old_sub_camera_view: None,
			old_viewport_size: None,
		},
		output_mode: CameraOutputMode::Write {
			blend_state: None,
			clear_color: ClearColorConfig::Default,
		},
	}
}
