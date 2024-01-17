pub mod player_animation_states;
pub mod player_movement;

pub trait ResourceKey
where
	Self: Sized,
{
	fn resource_keys() -> Iter<Self>;
	fn get_next(current: &Iter<Self>) -> Option<Self>;
	fn get_resource_path(value: &Self) -> String;
}

pub struct Iter<T>(pub Option<T>);

impl<TResourceKey: ResourceKey + Copy> Iterator for Iter<TResourceKey> {
	type Item = (TResourceKey, String);

	fn next(&mut self) -> Option<Self::Item> {
		let current = &self.0?;
		self.0 = TResourceKey::get_next(self);

		Some((*current, TResourceKey::get_resource_path(current)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Clone, Copy, PartialEq, Debug)]
	struct _MyType(usize);

	impl ResourceKey for _MyType {
		fn resource_keys() -> Iter<Self> {
			Iter(Some(_MyType(0)))
		}

		fn get_next(current: &Iter<Self>) -> Option<_MyType> {
			match &current.0?.0 {
				0 => Some(_MyType(1)),
				1 => Some(_MyType(200)),
				_ => None,
			}
		}

		fn get_resource_path(value: &Self) -> String {
			match &value.0 {
				0 => "ZERO".to_owned(),
				1 => "ONE".to_owned(),
				_ => "LAST".to_owned(),
			}
		}
	}

	#[test]
	fn iterate_keys() {
		assert_eq!(
			vec![
				(_MyType(0), "ZERO".to_owned()),
				(_MyType(1), "ONE".to_owned()),
				(_MyType(200), "LAST".to_owned())
			],
			_MyType::resource_keys().collect::<Vec<_>>()
		);
	}
}
