use crate::components::effect_material_data::EffectMaterialData;
use bevy::{
	ecs::component::Mutable,
	prelude::*,
	render::{
		render_resource::{AsBindGroup, ShaderType},
		storage::ShaderBuffer,
	},
	shader::ShaderRef,
};
use common::prelude::*;
use std::sync::LazyLock;

pub static NO_IMPACTS: LazyLock<Handle<ShaderBuffer>> = LazyLock::new(Handle::default);

#[derive(Asset, TypePath, AsBindGroup, Debug, PartialEq, Clone)]
pub(crate) struct EffectMaterial {
	#[texture(0)]
	#[sampler(1)]
	first_pass: Handle<Image>,
	#[uniform(2)]
	base_color: LinearRgba,
	#[uniform(3)]
	fresnel_color: LinearRgba,
	#[uniform(4)]
	flags: u32,
	#[storage(5, read_only)]
	impacts: Handle<ShaderBuffer>,
}

impl EffectMaterial {
	fn compute_impacts(impacts: &[Impact], transform: &GlobalTransform) -> ShaderBuffer {
		let inverse = transform.affine().inverse();
		let to_local = |Impact { position, strength }: &Impact| {
			let local = inverse.transform_point3(Vec3::from(position.0));
			let strength = **strength;

			LocalImpact { local, strength }
		};

		ShaderBuffer::from(impacts.iter().map(to_local).collect::<Vec<_>>())
	}

	fn local_empty_impacts(&mut self) {
		self.impacts = NO_IMPACTS.clone();
	}

	fn local_new_impacts(
		&mut self,
		buffers: &mut Assets<ShaderBuffer>,
		impacts: &[Impact],
		transform: &GlobalTransform,
	) {
		self.impacts = buffers.add(Self::compute_impacts(impacts, transform));
	}

	fn local_updated_impacts(
		&mut self,
		buffers: &mut Assets<ShaderBuffer>,
		impacts: &[Impact],
		transform: &GlobalTransform,
	) {
		if let Some(mut buffer) = buffers.get_mut(&self.impacts) {
			*buffer = Self::compute_impacts(impacts, transform);
		} else {
			self.impacts = buffers.add(Self::compute_impacts(impacts, transform));
		}
	}
}

impl Default for EffectMaterial {
	fn default() -> Self {
		Self {
			first_pass: default(),
			base_color: default(),
			fresnel_color: default(),
			flags: default(),
			impacts: NO_IMPACTS.clone(),
		}
	}
}

impl From<EffectMaterialData> for EffectMaterial {
	fn from(
		EffectMaterialData {
			first_pass,
			base_color,
			fresnel_color,
			flags,
			..
		}: EffectMaterialData,
	) -> Self {
		Self {
			first_pass,
			base_color,
			fresnel_color,
			flags,
			impacts: NO_IMPACTS.clone(),
		}
	}
}

impl Material for EffectMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/effect_shader.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Blend
	}

	fn enable_shadows() -> bool {
		false
	}
}

#[derive(Debug, PartialEq, ShaderType)]
pub(crate) struct LocalImpact {
	local: Vec3,
	strength: f32,
}

pub(crate) trait UpdateImpacts {
	type TBuffer: Resource<Mutability = Mutable>;

	fn update_impacts(
		&mut self,
		buffers: &mut Assets<ShaderBuffer>,
		impacts: &[Impact],
		transform: &GlobalTransform,
	);
}

impl UpdateImpacts for EffectMaterial {
	type TBuffer = Assets<ShaderBuffer>;

	fn update_impacts(
		&mut self,
		buffers: &mut Assets<ShaderBuffer>,
		impacts: &[Impact],
		transform: &GlobalTransform,
	) {
		match impacts {
			[] => self.local_empty_impacts(),
			_ if self.impacts == *NO_IMPACTS => self.local_new_impacts(buffers, impacts, transform),
			_ => self.local_updated_impacts(buffers, impacts, transform),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::{SingleThreadedApp, assert_count};
	use zyheeda_core::prelude::*;

	#[derive(Component, Debug, PartialEq)]
	struct _Material(EffectMaterial);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Assets<ShaderBuffer>>();
		app.add_plugins(MinimalPlugins);
		app.add_plugins(TransformPlugin);

		app
	}

	#[test]
	fn set_impacts() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				_Material(EffectMaterial::default()),
			))
			.id();
		let impacts = vec![Impact {
			position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
			strength: new_f32!(ImpactStrength(0.5)),
		}];

		app.world_mut().run_system_once(
			move |mut buffers: ResMut<Assets<ShaderBuffer>>,
			      mut transform: Query<(&GlobalTransform, &mut _Material)>| {
				let (transform, mut material) = transform.get_mut(entity).unwrap();
				material.0.update_impacts(&mut buffers, &impacts, transform);
			},
		)?;

		let mut materials = app.world_mut().query::<&_Material>();
		let [_Material(material)] = assert_count!(1, materials.iter(app.world()));
		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![LocalImpact {
					local: Vec3::new(1., 2., 3.),
					strength: 0.5
				}])
				.data
			),
			app.world()
				.resource::<Assets<ShaderBuffer>>()
				.get(&material.impacts)
				.map(|b| &b.data)
		);
		Ok(())
	}

	#[test]
	fn do_not_use_default_handle() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				_Material(EffectMaterial::default()),
			))
			.id();
		let impacts = vec![Impact {
			position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
			strength: new_f32!(ImpactStrength(0.5)),
		}];

		app.world_mut().run_system_once(
			move |mut buffers: ResMut<Assets<ShaderBuffer>>,
			      mut transform: Query<(&GlobalTransform, &mut _Material)>| {
				let (transform, mut material) = transform.get_mut(entity).unwrap();
				material.0.update_impacts(&mut buffers, &impacts, transform);
			},
		)?;

		let mut materials = app.world_mut().query::<&_Material>();
		let [_Material(material)] = assert_count!(1, materials.iter(app.world()));
		assert_ne!(&*NO_IMPACTS, &material.impacts);
		Ok(())
	}

	#[test]
	fn use_default_handle_when_impacts_empty() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				_Material(EffectMaterial::default()),
			))
			.id();

		app.world_mut().run_system_once(
			move |mut buffers: ResMut<Assets<ShaderBuffer>>,
			      mut transform: Query<(&GlobalTransform, &mut _Material)>| {
				let (transform, mut material) = transform.get_mut(entity).unwrap();
				material.0.update_impacts(&mut buffers, &[], transform);
			},
		)?;

		let mut materials = app.world_mut().query::<&_Material>();
		let [_Material(material)] = assert_count!(1, materials.iter(app.world()));
		assert_eq!(&*NO_IMPACTS, &material.impacts);
		Ok(())
	}

	#[test]
	fn use_default_handle_when_impacts_empty_after_some_impacts_were_set()
	-> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				_Material(EffectMaterial::default()),
			))
			.id();

		app.world_mut().run_system_once(
			move |mut buffers: ResMut<Assets<ShaderBuffer>>,
			      mut transform: Query<(&GlobalTransform, &mut _Material)>| {
				let (transform, mut material) = transform.get_mut(entity).unwrap();
				material.0.update_impacts(
					&mut buffers,
					&[Impact {
						position: GlobalVec3(VecNotNan::default()),
						strength: new_f32!(ImpactStrength(1.0)),
					}],
					transform,
				);
				material.0.update_impacts(&mut buffers, &[], transform);
			},
		)?;

		let mut materials = app.world_mut().query::<&_Material>();
		let [_Material(material)] = assert_count!(1, materials.iter(app.world()));
		assert_eq!(&*NO_IMPACTS, &material.impacts);
		Ok(())
	}

	#[test]
	fn reuse_handle() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				_Material(EffectMaterial::default()),
			))
			.id();

		let first_handle = app.world_mut().run_system_once(
			move |mut buffers: ResMut<Assets<ShaderBuffer>>,
			      mut transform: Query<(&GlobalTransform, &mut _Material)>| {
				let (transform, mut material) = transform.get_mut(entity).unwrap();
				material.0.update_impacts(
					&mut buffers,
					&[Impact {
						position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
						strength: new_f32!(ImpactStrength(0.5)),
					}],
					transform,
				);
				let first_handle = material.0.impacts.clone();
				material.0.update_impacts(
					&mut buffers,
					&[Impact {
						position: GlobalVec3(vec_not_nan!(2., 3., 4.)),
						strength: new_f32!(ImpactStrength(0.5)),
					}],
					transform,
				);
				first_handle
			},
		)?;

		let mut materials = app.world_mut().query::<&_Material>();
		let [_Material(material)] = assert_count!(1, materials.iter(app.world()));
		assert_eq!(&first_handle, &material.impacts);
		Ok(())
	}

	#[test]
	fn set_new_impacts() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				_Material(EffectMaterial::default()),
			))
			.id();
		let impacts = vec![Impact {
			position: GlobalVec3(vec_not_nan!(2., 3., 4.)),
			strength: new_f32!(ImpactStrength(0.5)),
		}];

		app.world_mut().run_system_once(
			move |mut buffers: ResMut<Assets<ShaderBuffer>>,
			      mut transform: Query<(&GlobalTransform, &mut _Material)>| {
				let (transform, mut material) = transform.get_mut(entity).unwrap();
				material.0.update_impacts(
					&mut buffers,
					&[Impact {
						position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
						strength: new_f32!(ImpactStrength(0.5)),
					}],
					transform,
				);
				material.0.update_impacts(&mut buffers, &impacts, transform);
			},
		)?;

		let mut materials = app.world_mut().query::<&_Material>();
		let [_Material(material)] = assert_count!(1, materials.iter(app.world()));
		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![LocalImpact {
					local: Vec3::new(2., 3., 4.),
					strength: 0.5
				}])
				.data
			),
			app.world()
				.resource::<Assets<ShaderBuffer>>()
				.get(&material.impacts)
				.map(|b| &b.data)
		);
		Ok(())
	}

	#[test]
	fn set_new_impacts_when_assets_data_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				_Material(EffectMaterial::default()),
			))
			.id();
		let impacts = vec![Impact {
			position: GlobalVec3(vec_not_nan!(2., 3., 4.)),
			strength: new_f32!(ImpactStrength(0.5)),
		}];

		app.world_mut().run_system_once(
			move |mut buffers: ResMut<Assets<ShaderBuffer>>,
			      mut transform: Query<(&GlobalTransform, &mut _Material)>| {
				let (transform, mut material) = transform.get_mut(entity).unwrap();
				material.0.update_impacts(
					&mut buffers,
					&[Impact {
						position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
						strength: new_f32!(ImpactStrength(0.5)),
					}],
					transform,
				);
				buffers.remove(&material.0.impacts);
				material.0.update_impacts(&mut buffers, &impacts, transform);
			},
		)?;

		let mut materials = app.world_mut().query::<&_Material>();
		let [_Material(material)] = assert_count!(1, materials.iter(app.world()));
		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![LocalImpact {
					local: Vec3::new(2., 3., 4.),
					strength: 0.5
				}])
				.data
			),
			app.world()
				.resource::<Assets<ShaderBuffer>>()
				.get(&material.impacts)
				.map(|b| &b.data)
		);
		Ok(())
	}

	#[test]
	fn translate_impacts_into_local() -> Result<(), RunSystemError> {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn((Transform::from_xyz(0., 1., 0.),))
			.id();
		let entity = app
			.world_mut()
			.spawn((
				ChildOf(parent),
				Transform::from_xyz(1., 0., 0.).looking_to(Dir3::Z, Dir3::Y),
				_Material(EffectMaterial::default()),
			))
			.id();
		app.update();

		app.world_mut().run_system_once(
			move |mut buffers: ResMut<Assets<ShaderBuffer>>,
			      mut transform: Query<(&GlobalTransform, &mut _Material)>| {
				let (transform, mut material) = transform.get_mut(entity).unwrap();
				material.0.update_impacts(
					&mut buffers,
					&[Impact {
						position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
						strength: new_f32!(ImpactStrength(0.5)),
					}],
					transform,
				);
			},
		)?;

		let mut materials = app.world_mut().query::<&_Material>();
		let [_Material(material)] = assert_count!(1, materials.iter(app.world()));
		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![LocalImpact {
					local: Vec3::new(0., 1., -3.),
					strength: 0.5
				}])
				.data
			),
			app.world()
				.resource::<Assets<ShaderBuffer>>()
				.get(&material.impacts)
				.map(|b| &b.data)
		);
		Ok(())
	}
}
