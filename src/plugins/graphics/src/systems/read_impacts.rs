use crate::components::effect_material_data::{EffectMaterialData, Impact};
use bevy::prelude::*;
use common::prelude::*;

impl EffectMaterialData {
	pub(crate) fn read_impacts<TImpactEvent>(
		impact: On<TImpactEvent>,
		mut effect_data: Query<(&mut Self, &GlobalTransform)>,
	) where
		TImpactEvent: EntityEvent + View<GlobalVec3>,
	{
		let impacted = impact.event_target();
		let position = impact.view();

		let Ok((mut data, transform)) = effect_data.get_mut(impacted) else {
			return;
		};

		let local = transform
			.affine()
			.inverse()
			.transform_point3(Vec3::from(*position));

		data.impacts.push(Impact::from_local(local));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(EntityEvent)]
	struct _ImpactEvent {
		entity: Entity,
		position: VecNotNan<3>,
	}

	impl View<GlobalVec3> for _ImpactEvent {
		fn view(&self) -> <GlobalVec3 as ViewField>::TValue<'_> {
			&self.position
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(EffectMaterialData::read_impacts::<_ImpactEvent>);

		app
	}

	#[test]
	fn set_impacts() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData::default())
			.trigger(|entity| _ImpactEvent {
				entity,
				position: vec_not_nan!(1., 2., 3.),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&EffectMaterialData {
				impacts: vec![Impact::from_local(Vec3::new(1., 2., 3.))],
				..default()
			}),
			app.world().entity(entity).get::<EffectMaterialData>()
		);
	}

	#[test]
	fn set_impacts_with_local_offset() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EffectMaterialData::default(),
				GlobalTransform::from(Transform::from_xyz(3., 5., 6.).looking_to(Dir3::Z, Dir3::Y)),
			))
			.trigger(|entity| _ImpactEvent {
				entity,
				position: vec_not_nan!(1., 2., 3.),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&EffectMaterialData {
				impacts: vec![Impact::from_local(Vec3::new(2., -3., 3.))],
				..default()
			}),
			app.world().entity(entity).get::<EffectMaterialData>()
		);
	}
}
