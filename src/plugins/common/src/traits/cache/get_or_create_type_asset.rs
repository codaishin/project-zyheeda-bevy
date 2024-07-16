use super::{GetOrCreateAsset, GetOrCreateTypeAsset};
use bevy::asset::{Asset, Handle};
use std::any::TypeId;

impl<T, TAsset> GetOrCreateTypeAsset<TAsset> for T
where
	T: GetOrCreateAsset<TypeId, TAsset>,
	TAsset: Asset,
{
	fn get_or_create_for<Key: 'static>(
		&mut self,
		create: impl FnMut() -> TAsset,
	) -> Handle<TAsset> {
		self.get_or_create(TypeId::of::<Key>(), create)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{asset::AssetId, color::Color, pbr::StandardMaterial, utils::default};
	use uuid::Uuid;

	#[derive(Default)]
	struct _GetOrCreateAsset {
		args: Vec<(TypeId, StandardMaterial)>,
		returns: Handle<StandardMaterial>,
	}

	impl GetOrCreateAsset<TypeId, StandardMaterial> for _GetOrCreateAsset {
		fn get_or_create(
			&mut self,
			key: TypeId,
			create: impl FnOnce() -> StandardMaterial,
		) -> Handle<StandardMaterial> {
			self.args.push((key, create()));
			self.returns.clone()
		}
	}

	fn as_get_or_create_type_asset(
		v: &mut impl GetOrCreateAsset<TypeId, StandardMaterial>,
	) -> &mut impl GetOrCreateTypeAsset<StandardMaterial> {
		v
	}

	#[test]
	fn use_returned_handle() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut cached = _GetOrCreateAsset {
			returns: handle.clone(),
			..default()
		};
		let cached = as_get_or_create_type_asset(&mut cached);

		assert_eq!(
			handle,
			cached.get_or_create_for::<u32>(StandardMaterial::default),
		)
	}

	#[test]
	fn call_get_or_create_with_proper_args() {
		let mut cached = _GetOrCreateAsset::default();

		as_get_or_create_type_asset(&mut cached).get_or_create_for::<u32>(|| StandardMaterial {
			base_color: Color::srgb(0., 1., 0.),
			..default()
		});

		assert_eq!(
			vec![(TypeId::of::<u32>(), Color::srgb(0., 1., 0.))],
			cached
				.args
				.into_iter()
				.map(|(t, m)| (t, m.base_color))
				.collect::<Vec<_>>(),
		)
	}
}
