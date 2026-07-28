use crate::{
	components::los::LoS,
	materials::lit_material::LitMaterial,
	resources::los_image::{LoSImageAtlas, LoSImageCubemap},
};
use bevy::{
	asset::UntypedAssetId,
	camera::{MainPassResolutionOverride, Viewport},
	core_pipeline::{
		Core3dSystems,
		core_3d::{Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey, main_opaque_pass_3d},
		schedule::Core3d,
	},
	ecs::system::{StaticSystemParam, SystemParamItem},
	material::descriptor::BindGroupLayoutDescriptor,
	mesh::MeshVertexBufferLayoutRef,
	pbr::{
		DrawMesh,
		MATERIAL_BIND_GROUP_INDEX,
		MeshPipeline,
		MeshPipelineKey,
		RenderMeshInstances,
		SetMeshBindGroup,
		SetMeshViewBindGroup,
		SetMeshViewBindingArrayBindGroup,
		ViewKeyCache,
	},
	platform::collections::HashSet,
	prelude::*,
	render::{
		Extract,
		Render,
		RenderApp,
		RenderDebugFlags,
		RenderSystems,
		batching::{
			GetBatchData,
			GetFullBatchData,
			gpu_preprocessing::{
				GpuPreprocessingMode,
				GpuPreprocessingSupport,
				UntypedPhaseIndirectParametersBuffers,
			},
		},
		camera::{DirtySpecializations, ExtractedCamera, PendingQueues},
		extract_component::ExtractComponentPlugin,
		extract_resource::{ExtractResource, ExtractResourcePlugin},
		mesh::{RenderMesh, allocator::MeshAllocator},
		render_asset::RenderAssets,
		render_phase::{
			AddRenderCommand,
			BinnedPhaseItem,
			BinnedRenderPhasePlugin,
			BinnedRenderPhaseType,
			CachedRenderPipelinePhaseItem,
			DrawError,
			DrawFunctionId,
			DrawFunctions,
			PhaseItem,
			PhaseItemExtraIndex,
			RenderCommand,
			RenderCommandResult,
			SetItemPipeline,
			ViewBinnedRenderPhases,
		},
		render_resource::{
			AsBindGroup,
			BindGroup,
			CachedRenderPipelineId,
			ColorTargetState,
			ColorWrites,
			Extent3d,
			Face,
			FragmentState,
			Origin3d,
			PipelineCache,
			PreparedBindGroup,
			PrimitiveState,
			RenderPassDescriptor,
			RenderPipelineDescriptor,
			SpecializedMeshPipeline,
			SpecializedMeshPipelineError,
			SpecializedMeshPipelines,
			StoreOp,
			TexelCopyTextureInfo,
			TextureAspect,
		},
		renderer::{RenderContext, RenderDevice, ViewQuery},
		sync_world::MainEntity,
		texture::GpuImage,
		view::{
			ExtractedView,
			NoIndirectDrawing,
			RenderVisibleEntities,
			RetainedViewEntity,
			ViewDepthTexture,
			ViewTarget,
		},
	},
	shader::ShaderDefVal,
};
use common::{
	errors::{ErrorData, Level},
	systems::log::OnError,
	zyheeda_commands::ZyheedaCommands,
};
use macros::asset_path;
use nonmax::NonMaxU32;
use std::{fmt::Display, ops::Range};

pub(crate) trait SetupDistancePipeline {
	fn setup_distance_pipeline(&mut self) -> &mut Self;
}

impl SetupDistancePipeline for App {
	fn setup_distance_pipeline(&mut self) -> &mut Self {
		self.add_plugins((
			ExtractResourcePlugin::<DistancePipelineData>::default(),
			ExtractResourcePlugin::<LoSImageAtlas>::default(),
			ExtractResourcePlugin::<LoSImageCubemap>::default(),
			ExtractComponentPlugin::<LoS>::default(),
			BinnedRenderPhasePlugin::<Distance, DistancePipeline>::new(RenderDebugFlags::default()),
		))
		.init_resource::<DistancePipelineData>();

		self.sub_app_mut(RenderApp)
			.init_resource::<DistancePipeline>()
			.init_resource::<DrawFunctions<Distance>>()
			.init_resource::<ViewBinnedRenderPhases<Distance>>()
			.init_resource::<SpecializedMeshPipelines<DistancePipelineSpecializer>>()
			.init_resource::<Pending>()
			.add_render_command::<Distance, DrawDistanceMaterial>()
			.add_systems(ExtractSchedule, DistancePipeline::extract_camera_phases)
			.add_systems(
				Render,
				(
					DistancePipelineDescriptor::init
						.run_if(not(resource_exists::<DistancePipelineDescriptor>)),
					DistancePipelineData::compute_bind_group,
					DistancePipeline::specialize_and_queue_draws.pipe(OnError::log),
				)
					.chain()
					.in_set(RenderSystems::QueueMeshes),
			)
			.add_systems(
				Core3d,
				(
					DistancePipeline::draw.pipe(OnError::log),
					copy_atlas_to_cubemap,
				)
					.after(main_opaque_pass_3d)
					.in_set(Core3dSystems::MainPass),
			);

		self
	}
}

fn copy_atlas_to_cubemap(
	view: ViewQuery<&LoS>,
	atlas: Res<LoSImageAtlas>,
	cubemap: Res<LoSImageCubemap>,
	images: Res<RenderAssets<GpuImage>>,
	mut render_context: RenderContext,
) {
	let los = view.into_inner();

	let Some(src) = images.get(&atlas.handle) else {
		return;
	};

	let Some(dst) = images.get(&cubemap.handle) else {
		return;
	};

	let offset = los.atlas_offset();
	let depth = los.cubemap_depth();

	render_context.command_encoder().copy_texture_to_texture(
		TexelCopyTextureInfo {
			texture: &src.texture,
			mip_level: 0,
			origin: Origin3d {
				x: offset.x,
				y: offset.y,
				z: 0,
			},
			aspect: TextureAspect::All,
		},
		TexelCopyTextureInfo {
			texture: &dst.texture,
			mip_level: 0,
			origin: Origin3d {
				x: 0,
				y: 0,
				z: depth,
			},
			aspect: TextureAspect::All,
		},
		Extent3d {
			width: LoS::SUB_VIEW,
			height: LoS::SUB_VIEW,
			depth_or_array_layers: 1,
		},
	);
}

#[derive(Resource)]
struct DistancePipeline {
	shader: Handle<Shader>,
}

impl DistancePipeline {
	const SHADER: &str = asset_path!("shaders/distance.wgsl");

	#[allow(clippy::type_complexity)]
	fn extract_camera_phases(
		mut phases: ResMut<ViewBinnedRenderPhases<Distance>>,
		cameras: Extract<Query<(Entity, &Camera, Has<NoIndirectDrawing>), With<LoS>>>,
		mut live_entities: Local<HashSet<RetainedViewEntity>>,
		gpu_preprocessing_support: Res<GpuPreprocessingSupport>,
	) {
		live_entities.clear();
		for (main_entity, camera, no_indirect_drawing) in &cameras {
			if !camera.is_active {
				continue;
			}

			let gpu_preprocessing_mode = gpu_preprocessing_support.min(if !no_indirect_drawing {
				GpuPreprocessingMode::Culling
			} else {
				GpuPreprocessingMode::PreprocessingOnly
			});

			let retained_view_entity = RetainedViewEntity::new(main_entity.into(), None, 0);

			phases.prepare_for_new_frame(retained_view_entity, gpu_preprocessing_mode);
			live_entities.insert(retained_view_entity);
		}

		phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
	}

	#[allow(clippy::too_many_arguments)]
	fn specialize_and_queue_draws(
		draw_functions: Res<DrawFunctions<Distance>>,
		pipeline_cache: Res<PipelineCache>,
		mesh_pipeline: Res<MeshPipeline>,
		distance_pipeline: Res<DistancePipeline>,
		distance_pipeline_descriptor: Res<DistancePipelineDescriptor>,
		allocator: Res<MeshAllocator>,
		meshes: Res<RenderAssets<RenderMesh>>,
		mesh_instances: Res<RenderMeshInstances>,
		view_key_cache: Res<ViewKeyCache>,
		dirty_specializations: Res<DirtySpecializations>,
		gpu_preprocessing_support: Res<GpuPreprocessingSupport>,
		mut specialized_pipelines: ResMut<SpecializedMeshPipelines<DistancePipelineSpecializer>>,
		mut phases: ResMut<ViewBinnedRenderPhases<Distance>>,
		mut views: Query<(&ExtractedView, &RenderVisibleEntities)>,
		mut pending: ResMut<Pending>,
	) -> Result<(), Vec<RenderError>> {
		let mut errors = vec![];
		let specializer = DistancePipelineSpecializer {
			shader: distance_pipeline.shader.clone(),
			mesh_pipeline: mesh_pipeline.clone(),
			descriptor: distance_pipeline_descriptor.descriptor.clone(),
		};

		for (view, visible_entities) in &mut views {
			let Some(phase) = phases.get_mut(&view.retained_view_entity) else {
				continue;
			};
			let Some(&view_key) = view_key_cache.get(&view.retained_view_entity) else {
				continue;
			};
			let Some(render_visible_mesh_entities) = visible_entities.get::<Mesh3d>() else {
				continue;
			};

			let pending = pending.prepare_for_new_frame(view.retained_view_entity);
			let draw_function = draw_functions.read().id::<DrawDistanceMaterial>();
			let dequeue_entities = dirty_specializations
				.iter_to_dequeue(view.retained_view_entity, render_visible_mesh_entities);
			let render_entities = dirty_specializations.iter_to_queue(
				view.retained_view_entity,
				render_visible_mesh_entities,
				&pending.prev_frame,
			);

			for &main_entity in dequeue_entities {
				phase.remove(main_entity);
			}

			for (entity, main_entity) in render_entities {
				let Some(instance) = mesh_instances.render_mesh_queue_data(*main_entity) else {
					pending.current_frame.insert((*entity, *main_entity));
					continue;
				};

				let id = instance.mesh_asset_id();

				let Some(mesh) = meshes.get(id) else {
					errors.push(RenderError::NoMesh(id));
					continue;
				};
				let Some(slabs) = allocator.mesh_slabs(&id) else {
					errors.push(RenderError::NoSlabs(id));
					continue;
				};

				let pipeline_id = specialized_pipelines.specialize(
					&pipeline_cache,
					&specializer,
					view_key
						| MeshPipelineKey::from_primitive_topology_and_strip_index(
							mesh.primitive_topology(),
							mesh.index_format(),
						),
					&mesh.layout,
				);
				let pipeline = match pipeline_id {
					Ok(id) => id,
					Err(err) => {
						errors.push(RenderError::Specialize(err));
						continue;
					}
				};

				phase.add(
					Opaque3dBatchSetKey {
						pipeline,
						draw_function,
						material_bind_group_index: Some(MATERIAL_BIND_GROUP_INDEX as u32),
						slabs,
						lightmap_slab: instance.shared.lightmap_slab_index().map(|index| *index),
					},
					Opaque3dBinKey {
						asset_id: UntypedAssetId::from(id),
					},
					(Entity::PLACEHOLDER, *main_entity),
					instance.current_uniform_index,
					BinnedRenderPhaseType::mesh(
						instance.should_batch(),
						&gpu_preprocessing_support,
					),
				);
			}
		}

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}

	#[allow(clippy::type_complexity)]
	fn draw(
		world: &World,
		view: ViewQuery<
			(
				&ExtractedCamera,
				&ExtractedView,
				&ViewTarget,
				&ViewDepthTexture,
				Option<&MainPassResolutionOverride>,
			),
			With<LoS>,
		>,
		phases: Res<ViewBinnedRenderPhases<Distance>>,
		mut ctx: RenderContext,
	) -> Result<(), RenderError> {
		let entity = view.entity();
		let (camera, view, target, depth, resolution_override) = view.into_inner();

		let Some(phase) = phases.get(&view.retained_view_entity) else {
			return Ok(());
		};
		let depth_stencil_attachment = Some(depth.get_attachment(StoreOp::Store));
		let mut render_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
			label: Some("distance pass"),
			color_attachments: &[Some(target.get_color_attachment())],
			depth_stencil_attachment,
			..default()
		});

		let viewport = camera.viewport.as_ref();
		let viewport = Viewport::from_viewport_and_override(viewport, resolution_override);
		if let Some(viewport) = viewport {
			render_pass.set_camera_viewport(&viewport);
		}

		phase
			.render(&mut render_pass, world, entity)
			.map_err(RenderError::Draw)
	}
}

impl FromWorld for DistancePipeline {
	fn from_world(world: &mut World) -> Self {
		Self {
			shader: world.resource::<AssetServer>().load(Self::SHADER),
		}
	}
}

impl GetBatchData for DistancePipeline {
	type Param = <MeshPipeline as GetBatchData>::Param;
	type BatchSetCompareData = <MeshPipeline as GetBatchData>::BatchSetCompareData;
	type BatchCompareData = <MeshPipeline as GetBatchData>::BatchCompareData;
	type BufferData = <MeshPipeline as GetBatchData>::BufferData;

	fn get_batch_data(
		param: &SystemParamItem<Self::Param>,
		query_item: (Entity, MainEntity),
	) -> Option<(
		Self::BufferData,
		Option<(Self::BatchSetCompareData, Self::BatchCompareData)>,
	)> {
		MeshPipeline::get_batch_data(param, query_item)
	}
}

impl GetFullBatchData for DistancePipeline {
	type BufferInputData = <MeshPipeline as GetFullBatchData>::BufferInputData;

	fn get_index_and_compare_data(
		param: &SystemParamItem<Self::Param>,
		query_item: MainEntity,
	) -> Option<(
		NonMaxU32,
		Option<(Self::BatchSetCompareData, Self::BatchCompareData)>,
	)> {
		MeshPipeline::get_index_and_compare_data(param, query_item)
	}

	fn get_binned_batch_data(
		param: &SystemParamItem<Self::Param>,
		query_item: MainEntity,
	) -> Option<Self::BufferData> {
		MeshPipeline::get_binned_batch_data(param, query_item)
	}

	fn write_batch_indirect_parameters_metadata(
		indexed: bool,
		base_output_index: u32,
		batch_set_index: Option<NonMaxU32>,
		indirect_parameters_buffers: &mut UntypedPhaseIndirectParametersBuffers,
		indirect_parameters_offset: u32,
	) {
		MeshPipeline::write_batch_indirect_parameters_metadata(
			indexed,
			base_output_index,
			batch_set_index,
			indirect_parameters_buffers,
			indirect_parameters_offset,
		);
	}

	fn get_binned_index(
		param: &SystemParamItem<Self::Param>,
		query_item: MainEntity,
	) -> Option<NonMaxU32> {
		MeshPipeline::get_binned_index(param, query_item)
	}
}

struct DistancePipelineSpecializer {
	shader: Handle<Shader>,
	mesh_pipeline: MeshPipeline,
	descriptor: BindGroupLayoutDescriptor,
}

impl SpecializedMeshPipeline for DistancePipelineSpecializer {
	type Key = MeshPipelineKey;

	fn specialize(
		&self,
		key: Self::Key,
		layout: &MeshVertexBufferLayoutRef,
	) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
		let mut base_descriptor = self.mesh_pipeline.specialize(key, layout)?;
		base_descriptor.layout.push(self.descriptor.clone());

		Ok(RenderPipelineDescriptor {
			label: Some("Distance Pipeline".into()),
			layout: base_descriptor.layout,
			vertex: base_descriptor.vertex,
			fragment: Some(FragmentState {
				shader: self.shader.clone(),
				targets: vec![Some(ColorTargetState {
					format: key.target_format(),
					blend: None,
					write_mask: ColorWrites::ALL,
				})],
				shader_defs: vec![ShaderDefVal::Int(
					"MATERIAL_BIND_GROUP".to_owned(),
					MATERIAL_BIND_GROUP_INDEX as i32,
				)],
				..default()
			}),
			primitive: PrimitiveState {
				topology: key.primitive_topology(),
				strip_index_format: key.strip_index_format(),
				cull_mode: Some(Face::Back),
				..default()
			},
			depth_stencil: base_descriptor.depth_stencil,
			..default()
		})
	}
}

struct Distance(Opaque3d);

impl PhaseItem for Distance {
	#[inline]
	fn entity(&self) -> Entity {
		self.0.entity()
	}

	#[inline]
	fn main_entity(&self) -> MainEntity {
		self.0.main_entity()
	}

	#[inline]
	fn draw_function(&self) -> DrawFunctionId {
		self.0.draw_function()
	}

	#[inline]
	fn batch_range(&self) -> &Range<u32> {
		self.0.batch_range()
	}

	#[inline]
	fn batch_range_mut(&mut self) -> &mut Range<u32> {
		self.0.batch_range_mut()
	}

	#[inline]
	fn extra_index(&self) -> PhaseItemExtraIndex {
		self.0.extra_index()
	}

	#[inline]
	fn batch_range_and_extra_index_mut(&mut self) -> (&mut Range<u32>, &mut PhaseItemExtraIndex) {
		self.0.batch_range_and_extra_index_mut()
	}
}

impl BinnedPhaseItem for Distance {
	type BinKey = <Opaque3d as BinnedPhaseItem>::BinKey;
	type BatchSetKey = <Opaque3d as BinnedPhaseItem>::BatchSetKey;

	fn new(
		batch_set_key: Self::BatchSetKey,
		bin_key: Self::BinKey,
		representative_entity: (Entity, MainEntity),
		batch_range: Range<u32>,
		extra_index: PhaseItemExtraIndex,
	) -> Self {
		Self(Opaque3d::new(
			batch_set_key,
			bin_key,
			representative_entity,
			batch_range,
			extra_index,
		))
	}
}

impl CachedRenderPipelinePhaseItem for Distance {
	#[inline]
	fn cached_pipeline(&self) -> CachedRenderPipelineId {
		self.0.cached_pipeline()
	}
}

type DrawDistanceMaterial = (
	SetItemPipeline,
	SetMeshViewBindGroup<0>,
	SetMeshViewBindingArrayBindGroup<1>,
	SetMeshBindGroup<2>,
	SetDistanceBindGroup<MATERIAL_BIND_GROUP_INDEX>,
	DrawMesh,
);

struct SetDistanceBindGroup<const I: usize>;

impl<P, const I: usize> RenderCommand<P> for SetDistanceBindGroup<I>
where
	P: PhaseItem,
{
	type Param = Res<'static, DistancePipelineData>;
	type ViewQuery = ();
	type ItemQuery = ();

	fn render<'w>(
		_: &P,
		_: bevy::ecs::query::ROQueryItem<'w, '_, Self::ViewQuery>,
		_: Option<bevy::ecs::query::ROQueryItem<'w, '_, Self::ItemQuery>>,
		data: SystemParamItem<'w, '_, Self::Param>,
		pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
	) -> bevy::render::render_phase::RenderCommandResult {
		let data = data.into_inner();

		match data.bind_group {
			None => RenderCommandResult::Failure("Distance material bind group missing"),
			Some(ref bind_group) => {
				pass.set_bind_group(I, bind_group, &[]);

				RenderCommandResult::Success
			}
		}
	}
}

impl ErrorData for RenderError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Distance render error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[derive(Resource, Debug, PartialEq)]
struct DistancePipelineDescriptor {
	descriptor: BindGroupLayoutDescriptor,
}

impl DistancePipelineDescriptor {
	fn init(mut commands: ZyheedaCommands, render_device: Res<RenderDevice>) {
		let descriptor = DistanceMaterial::bind_group_layout_descriptor(&render_device);

		commands.insert_resource(Self { descriptor });
	}
}

#[derive(Resource, ExtractResource, Debug, PartialEq, Default, Clone)]
pub(crate) struct DistancePipelineData {
	pub(crate) player_position: Vec3,
	bind_group: Option<BindGroup>,
}

impl DistancePipelineData {
	fn compute_bind_group(
		mut data: ResMut<Self>,
		render_device: Res<RenderDevice>,
		pipeline_cache: Res<PipelineCache>,
		descriptor: Res<DistancePipelineDescriptor>,
		param: StaticSystemParam<<DistanceMaterial as AsBindGroup>::Param>,
	) {
		let mut param = param.into_inner();
		let mat = DistanceMaterial {
			player_position: data.player_position,
			falloff: LitMaterial::LINEAR_FALLOFF,
		};

		data.bind_group = mat
			.as_bind_group(
				&descriptor.descriptor,
				&render_device,
				&pipeline_cache,
				&mut param,
			)
			.map(|PreparedBindGroup { bind_group, .. }| bind_group)
			.ok()
	}
}

#[derive(AsBindGroup)]
struct DistanceMaterial {
	#[uniform(0)]
	player_position: Vec3,
	#[uniform(1)]
	falloff: f32,
}

#[derive(Default, Deref, DerefMut, Resource)]
struct Pending(pub PendingQueues);

#[derive(Debug)]
enum RenderError {
	NoMesh(AssetId<Mesh>),
	NoSlabs(AssetId<Mesh>),
	Specialize(SpecializedMeshPipelineError),
	Draw(DrawError),
}

impl Display for RenderError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			RenderError::NoMesh(id) => write!(f, "{id}: no mesh"),
			RenderError::NoSlabs(id) => write!(f, "{id}: no slabs"),
			RenderError::Specialize(e) => write!(f, "Specialization failed with: {e}"),
			RenderError::Draw(e) => write!(f, "Draw failed with: {e}"),
		}
	}
}
