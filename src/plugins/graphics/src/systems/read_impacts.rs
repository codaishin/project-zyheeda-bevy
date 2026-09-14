use crate::components::effect_material_data::EffectMaterialData;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::prelude::*;

impl EffectMaterialData {
	pub(crate) fn read_impacts<TImpacts>(
		impacted: StaticSystemParam<TImpacts>,
		effect_data: Query<(Entity, &mut Self)>,
	) where
		TImpacts: for<'c> TryGetContext<Impacted, TContext<'c>: IterImpacts>,
	{
		for (entity, mut data) in effect_data {
			let key = Impacted { entity };
			let Some(impacted) = TImpacts::try_get_context(&impacted, key) else {
				continue;
			};

			if !impacted.context_changed() {
				continue;
			}

			data.impacts = impacted.iter_impacts().collect();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;
	use zyheeda_core::prelude::*;

	#[derive(Component)]
	struct _Impacted(Vec<Impact>);

	impl IterImpacts for _Impacted {
		type TIter<'a>
			= std::iter::Copied<std::slice::Iter<'a, Impact>>
		where
			Self: 'a;

		fn iter_impacts(&self) -> Self::TIter<'_> {
			self.0.iter().copied()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			EffectMaterialData::read_impacts::<Query<Ref<_Impacted>>>,
		);

		app
	}

	#[test]
	fn set_impacts() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EffectMaterialData::default(),
				_Impacted(vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.6)),
				}]),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.6)),
				}],
				..default()
			}),
			app.world().entity(entity).get::<EffectMaterialData>()
		);
	}

	#[test]
	fn act_oly_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EffectMaterialData::default(),
				_Impacted(vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.6)),
				}]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(EffectMaterialData::default());
		app.update();

		assert_eq!(
			Some(&EffectMaterialData::default()),
			app.world().entity(entity).get::<EffectMaterialData>()
		);
	}

	#[test]
	fn act_again_if_context_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EffectMaterialData::default(),
				_Impacted(vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.6)),
				}]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(EffectMaterialData::default())
			.get_mut::<_Impacted>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.6)),
				}],
				..default()
			}),
			app.world().entity(entity).get::<EffectMaterialData>()
		);
	}
}
