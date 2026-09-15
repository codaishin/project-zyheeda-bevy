use crate::components::effect_material_data::{EffectMaterialData, Impact, ImpactStrength};
use bevy::prelude::*;
use std::time::Duration;

impl EffectMaterialData {
	pub(crate) fn decay_impacts(
		DecayPerSecond(decay): DecayPerSecond,
	) -> impl IntoSystem<In<Duration>, (), ()> {
		IntoSystem::into_system(move |In(delta): In<Duration>, data: Query<&mut Self>| {
			let delta_secs = delta.as_secs_f32();

			for mut data in data {
				if data.impacts.is_empty() {
					continue;
				}

				data.impacts.retain_mut(|Impact { strength, .. }| {
					let new_strength = **strength - *decay * delta_secs;

					let Ok(new_strength) = ImpactStrength::try_from_f32(new_strength) else {
						return false;
					};

					*strength = new_strength;

					true
				});
			}
		})
	}
}

pub(crate) struct DecayPerSecond(pub(crate) ImpactStrength);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::effect_material_data::Impact;
	use common::vec_not_nan;
	use testing::{IsChanged, SingleThreadedApp};
	use zyheeda_core::new_f32;

	fn setup(delta: Duration, decay: DecayPerSecond) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				(move || delta).pipe(EffectMaterialData::decay_impacts(decay)),
				IsChanged::<EffectMaterialData>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn decay_strength() {
		let mut app = setup(
			Duration::from_secs(1),
			DecayPerSecond(new_f32!(ImpactStrength(0.4))),
		);
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData {
				impacts: vec![Impact {
					global: vec_not_nan!(1., 2., 3.),
					strength: new_f32!(ImpactStrength(1.)),
				}],
				..default()
			})
			.id();

		app.update();

		assert_eq!(
			Some(&EffectMaterialData {
				impacts: vec![Impact {
					global: vec_not_nan!(1., 2., 3.),
					strength: new_f32!(ImpactStrength(0.6)),
				}],
				..default()
			}),
			app.world().entity(entity).get::<EffectMaterialData>(),
		);
	}

	#[test]
	fn decay_strength_scaled_by_delta() {
		let mut app = setup(
			Duration::from_millis(500),
			DecayPerSecond(new_f32!(ImpactStrength(0.4))),
		);
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData {
				impacts: vec![Impact {
					global: vec_not_nan!(1., 2., 3.),
					strength: new_f32!(ImpactStrength(1.)),
				}],
				..default()
			})
			.id();

		app.update();

		assert_eq!(
			Some(&EffectMaterialData {
				impacts: vec![Impact {
					global: vec_not_nan!(1., 2., 3.),
					strength: new_f32!(ImpactStrength(0.8)),
				}],
				..default()
			}),
			app.world().entity(entity).get::<EffectMaterialData>(),
		);
	}

	#[test]
	fn remove_if_impact_strength_would_be_invalid() {
		let mut app = setup(
			Duration::from_secs(1),
			DecayPerSecond(new_f32!(ImpactStrength(1.))),
		);
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData {
				impacts: vec![Impact {
					global: vec_not_nan!(1., 2., 3.),
					strength: new_f32!(ImpactStrength(1.)),
				}],
				..default()
			})
			.id();

		app.update();

		assert_eq!(
			Some(&EffectMaterialData {
				impacts: vec![],
				..default()
			}),
			app.world().entity(entity).get::<EffectMaterialData>(),
		);
	}

	#[test]
	fn leave_unchanged_if_empty() {
		let mut app = setup(
			Duration::from_secs(1),
			DecayPerSecond(new_f32!(ImpactStrength(0.4))),
		);
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData {
				impacts: vec![],
				..default()
			})
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(entity)
				.get::<IsChanged<EffectMaterialData>>(),
		);
	}
}
