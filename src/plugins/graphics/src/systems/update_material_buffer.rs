use bevy::{ecs::component::Mutable, prelude::*};
use common::traits::accessors::get::View;

impl<T> UpdateMaterialBuffer for T where T: Component<Mutability = Mutable> {}

pub(crate) trait UpdateMaterialBuffer: Component<Mutability = Mutable> + Sized {
	fn update_material_buffer<TMaterial>(
		data: Query<(&Self, &GlobalTransform), Changed<Self>>,
		mut materials: ResMut<Assets<TMaterial>>,
		mut buffers: ResMut<TMaterial::TBuffer>,
	) where
		TMaterial: Asset + UpdateMaterial<TData = Self>,
		Self: View<Handle<TMaterial>>,
	{
		for (data, transform) in data {
			let Some(mut material) = materials.get_mut(data.view()) else {
				continue;
			};

			material.update_material(&mut buffers, data, transform);
		}
	}
}

pub(crate) trait UpdateMaterial {
	type TData: Component;
	type TBuffer: Resource<Mutability = Mutable>;

	fn update_material(
		&mut self,
		buffers: &mut Self::TBuffer,
		data: &Self::TData,
		transform: &GlobalTransform,
	);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::render::render_resource::AsBindGroup;
	use testing::{SingleThreadedApp, assert_some, new_handle};

	#[derive(Component, Debug, PartialEq, Clone)]
	#[require(GlobalTransform)]
	struct _Data(Handle<_Material>);

	impl View<Handle<_Material>> for _Data {
		fn view(&self) -> &'_ Handle<_Material> {
			&self.0
		}
	}

	#[derive(Asset, TypePath, AsBindGroup, Debug, PartialEq, Default, Clone)]
	struct _Material {
		data: Option<_Data>,
		transform: Option<GlobalTransform>,
	}

	impl _Material {
		fn clear(&mut self) {
			self.data = None;
			self.transform = None;
		}
	}

	impl Material for _Material {}

	impl UpdateMaterial for _Material {
		type TData = _Data;
		type TBuffer = _Buffers;

		fn update_material(
			&mut self,
			_: &mut Self::TBuffer,
			data: &_Data,
			transform: &GlobalTransform,
		) {
			self.data = Some(data.clone());
			self.transform = Some(*transform);
		}
	}

	#[derive(Resource, Default)]
	struct _Buffers;

	fn setup<const N: usize>(materials: [&Handle<_Material>; N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		for id in materials {
			_ = assets.insert(
				id,
				_Material {
					data: None,
					transform: None,
				},
			);
		}

		app.init_resource::<_Buffers>();
		app.insert_resource(assets);
		app.add_systems(Update, _Data::update_material_buffer::<_Material>);

		app
	}

	#[test]
	fn updated_from_correct_data() {
		let handle = new_handle();
		let data = _Data(handle.clone());
		let mut app = setup([&handle]);
		app.world_mut()
			.spawn((data.clone(), GlobalTransform::from_xyz(1., 2., 3.)));

		app.update();

		assert_eq!(
			Some(&_Material {
				data: Some(data),
				transform: Some(GlobalTransform::from_xyz(1., 2., 3.))
			}),
			app.world().resource::<Assets<_Material>>().get(&handle),
		);
	}

	#[test]
	fn act_only_once() {
		let handle = new_handle();
		let data = _Data(handle.clone());
		let mut app = setup([&handle]);
		app.world_mut().spawn(data.clone());

		app.update();
		let mut materials = app.world_mut().resource_mut::<Assets<_Material>>();
		assert_some!(materials.get_mut(&handle)).clear();
		app.update();

		assert_eq!(
			Some(&_Material {
				data: None,
				transform: None
			}),
			app.world().resource::<Assets<_Material>>().get(&handle),
		);
	}

	#[test]
	fn act_again_of_data_changed() {
		let handle = new_handle();
		let data = _Data(handle.clone());
		let mut app = setup([&handle]);
		let entity = app.world_mut().spawn(data.clone()).id();

		app.update();
		let mut materials = app.world_mut().resource_mut::<Assets<_Material>>();
		assert_some!(materials.get_mut(&handle)).data = None;
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Data>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&_Material {
				data: Some(data),
				transform: Some(GlobalTransform::default())
			}),
			app.world().resource::<Assets<_Material>>().get(&handle),
		);
	}
}
