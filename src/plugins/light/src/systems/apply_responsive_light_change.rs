use crate::components::{ChangeLight, ResponsiveLight};
use bevy::{
	ecs::{
		entity::Entity,
		system::{Commands, Query, Res},
	},
	pbr::PointLight,
	render::view::Visibility,
	time::Time,
};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};
use std::{ops::Deref, time::Duration};

#[derive(PartialEq)]
enum State {
	Done,
	Busy,
}

pub(crate) fn apply_responsive_light_change<TTime: Sync + Send + 'static + Default>(
	mut commands: Commands,
	mut lights: Query<&mut PointLight>,
	changes: Query<(Entity, &ChangeLight)>,
	time: Res<Time<TTime>>,
) {
	let delta = time.delta();
	for (id, change) in &changes {
		let state = apply_change(&mut commands, &mut lights, change, delta);
		remove_change_component(&mut commands, state, id);
	}
}

fn apply_change(
	commands: &mut Commands,
	lights: &mut Query<&mut PointLight>,
	change: &ChangeLight,
	delta: Duration,
) -> State {
	match change {
		ChangeLight::Increase(light) => increase(commands, lights, light, delta),
		ChangeLight::Decrease(light) => decrease(commands, lights, light, delta),
	}
}

fn remove_change_component(commands: &mut Commands, state: State, id: Entity) {
	if state != State::Done {
		return;
	}
	commands.try_remove_from::<ChangeLight>(id);
}

fn increase(
	commands: &mut Commands,
	lights: &mut Query<&mut PointLight>,
	light: &ResponsiveLight,
	delta: Duration,
) -> State {
	let Ok(mut target_light) = lights.get_mut(light.light) else {
		return State::Busy;
	};

	if target_light.intensity == 0. {
		commands.try_insert_on(light.light, Visibility::Visible);
		commands.try_insert_on(light.model, light.data.light_on_material.clone());
	}

	let change = *light.data.change.deref() * delta.as_secs_f32();
	let max = *light.data.max.deref();
	if max - target_light.intensity > change {
		target_light.intensity += change;
		return State::Busy;
	}

	target_light.intensity = max;

	State::Done
}

fn decrease(
	commands: &mut Commands,
	lights: &mut Query<&mut PointLight>,
	light: &ResponsiveLight,
	delta: Duration,
) -> State {
	let Ok(mut target_light) = lights.get_mut(light.light) else {
		return State::Busy;
	};

	let change = *light.data.change.deref() * delta.as_secs_f32();
	if change < target_light.intensity {
		target_light.intensity -= change;
		return State::Busy;
	}

	target_light.intensity = 0.;
	commands.try_insert_on(light.model, light.data.light_off_material.clone());
	commands.try_insert_on(light.light, Visibility::Hidden);

	State::Done
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{ResponsiveLight, ResponsiveLightData};
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Handle},
		pbr::StandardMaterial,
		render::view::Visibility,
		time::{Real, Time},
		utils::default,
	};
	use common::{
		test_tools::utils::{SingleThreadedApp, TickTime},
		tools::{Intensity, IntensityChangePerSecond, Units},
		traits::clamp_zero_positive::ClampZeroPositive,
	};
	use std::time::Duration;
	use uuid::Uuid;

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Time<Real>>();
		app.add_systems(Update, apply_responsive_light_change::<Real>);

		app
	}

	#[test]
	fn increase_light_intensity() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(11.),
			},
		};
		app.world_mut().spawn(ChangeLight::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(53., light.intensity);
	}

	#[test]
	fn increase_light_intensity_scaled_by_delta() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(11.),
			},
		};
		app.world_mut().spawn(ChangeLight::Increase(responsive));

		app.tick_time(Duration::from_millis(100));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(43.1, light.intensity);
	}

	#[test]
	fn increase_light_intensity_clamped_at_max() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(200.),
			},
		};
		app.world_mut().spawn(ChangeLight::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(100., light.intensity);
	}

	#[test]
	fn insert_light_visibility_on_increase() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 0.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(11.),
			},
		};
		app.world_mut().spawn(ChangeLight::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light);

		assert_eq!(Some(&Visibility::Visible), light.get::<Visibility>());
	}

	#[test]
	fn do_not_insert_light_visibility_on_increase_when_intensity_not_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 1.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(11.),
			},
		};
		app.world_mut().spawn(ChangeLight::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light);

		assert_eq!(None, light.get::<Visibility>());
	}

	#[test]
	fn set_light_on_material_when_increasing() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 0.,
				..default()
			})
			.id();
		let light_on_material = new_handle();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: light_on_material.clone(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(11.),
			},
		};
		app.world_mut().spawn(ChangeLight::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let model = app.world().entity(model);

		assert_eq!(
			Some(&light_on_material),
			model.get::<Handle<StandardMaterial>>()
		);
	}

	#[test]
	fn do_not_set_light_on_material_when_intensity_not_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 1.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(11.),
			},
		};
		app.world_mut().spawn(ChangeLight::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let model = app.world().entity(model);

		assert_eq!(None, model.get::<Handle<StandardMaterial>>());
	}

	#[test]
	fn remove_change_light_when_reached_max() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(58.),
			},
		};
		let responsive = app
			.world_mut()
			.spawn(ChangeLight::Increase(responsive))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		let responsive = app.world().entity(responsive);

		assert_eq!(None, responsive.get::<ChangeLight>());
	}

	#[test]
	fn do_not_remove_change_light_when_not_reached_max() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(57.),
			},
		};
		let responsive_entity = app
			.world_mut()
			.spawn(ChangeLight::Increase(responsive.clone()))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		let responsive_entity = app.world().entity(responsive_entity);

		assert_eq!(
			Some(&ChangeLight::Increase(responsive)),
			responsive_entity.get::<ChangeLight>()
		);
	}

	#[test]
	fn decrease_light_intensity() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(11.),
			},
		};
		app.world_mut().spawn(ChangeLight::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(31., light.intensity);
	}

	#[test]
	fn decrease_light_intensity_by_delta() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(11.),
			},
		};
		app.world_mut().spawn(ChangeLight::Decrease(responsive));

		app.tick_time(Duration::from_millis(100));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(40.9, light.intensity);
	}

	#[test]
	fn set_light_off_material_when_decreasing_to_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let light_off_material = new_handle();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: light_off_material.clone(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(42.),
			},
		};
		app.world_mut().spawn(ChangeLight::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let model = app.world().entity(model);

		assert_eq!(
			Some(&light_off_material),
			model.get::<Handle<StandardMaterial>>()
		);
	}

	#[test]
	fn do_not_set_light_off_material_when_decreasing_above_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let light_off_material = new_handle();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: light_off_material.clone(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(41.),
			},
		};
		app.world_mut().spawn(ChangeLight::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let model = app.world().entity(model);

		assert_eq!(None, model.get::<Handle<StandardMaterial>>());
	}

	#[test]
	fn set_light_off_material_when_decreasing_to_below_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let light_off_material = new_handle();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: light_off_material.clone(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(43.),
			},
		};
		app.world_mut().spawn(ChangeLight::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let model = app.world().entity(model);

		assert_eq!(
			Some(&light_off_material),
			model.get::<Handle<StandardMaterial>>()
		);
	}

	#[test]
	fn decrease_light_intensity_clamped_at_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(43.),
			},
		};
		app.world_mut().spawn(ChangeLight::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(0., light.intensity);
	}

	#[test]
	fn remove_change_light_when_reached_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(42.),
			},
		};
		let responsive = app
			.world_mut()
			.spawn(ChangeLight::Decrease(responsive))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		let responsive = app.world().entity(responsive);

		assert_eq!(None, responsive.get::<ChangeLight>());
	}

	#[test]
	fn do_not_remove_change_light_when_not_reached_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(41.),
			},
		};
		let responsive_entity = app
			.world_mut()
			.spawn(ChangeLight::Decrease(responsive.clone()))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		let responsive_entity = app.world().entity(responsive_entity);

		assert_eq!(
			Some(&ChangeLight::Decrease(responsive)),
			responsive_entity.get::<ChangeLight>()
		);
	}

	#[test]
	fn insert_light_hidden_when_reaching_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(42.),
			},
		};
		app.world_mut().spawn(ChangeLight::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light);

		assert_eq!(Some(&Visibility::Hidden), light.get::<Visibility>());
	}

	#[test]
	fn do_not_insert_light_hidden_when_not_reaching_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			data: ResponsiveLightData {
				range: Units::new(0.),
				light_on_material: new_handle(),
				light_off_material: new_handle(),
				max: Intensity::new(100.),
				change: IntensityChangePerSecond::new(41.),
			},
		};
		app.world_mut().spawn(ChangeLight::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light);

		assert_eq!(None, light.get::<Visibility>());
	}
}
