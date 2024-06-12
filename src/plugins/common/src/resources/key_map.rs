use crate::traits::{
	iteration::IterFinite,
	map_value::{MapForward, TryMapBackwards},
};
use bevy::ecs::system::Resource;
use std::marker::PhantomData;

//FIXME: Add the possibility to override defaults
#[derive(Resource)]
pub struct KeyMap<TKey, TValue>
where
	TKey: IterFinite + Copy,
	TValue: From<TKey> + PartialEq,
{
	phantom_data: PhantomData<(TKey, TValue)>,
}

impl<TKey, TValue> Default for KeyMap<TKey, TValue>
where
	TKey: IterFinite + Copy,
	TValue: From<TKey> + PartialEq,
{
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
		}
	}
}

impl<TKey, TValue> MapForward<TKey, TValue> for KeyMap<TKey, TValue>
where
	TKey: IterFinite + Copy,
	TValue: From<TKey> + PartialEq,
{
	fn map_forward(&self, value: TKey) -> TValue {
		TValue::from(value)
	}
}

impl<TKey, TValue> TryMapBackwards<TValue, TKey> for KeyMap<TKey, TValue>
where
	TKey: IterFinite + Copy,
	TValue: From<TKey> + PartialEq,
{
	fn try_map_backwards(&self, value: TValue) -> Option<TKey> {
		TKey::iterator().find(|key| value == TValue::from(*key))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::iteration::{Iter, IterFinite};

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _From {
		Small,
		Big,
	}

	#[derive(Debug, PartialEq)]
	struct _To(&'static str);

	impl From<_From> for _To {
		fn from(value: _From) -> Self {
			match value {
				_From::Small => _To("small"),
				_From::Big => _To("big"),
			}
		}
	}

	impl IterFinite for _From {
		fn iterator() -> Iter<Self> {
			Iter(Some(_From::Small))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_From::Small => Some(_From::Big),
				_From::Big => None,
			}
		}
	}

	#[test]
	fn map_forwards() {
		let mapper = KeyMap::<_From, _To>::default();
		let mapped = mapper.map_forward(_From::Big);

		assert_eq!(_To("big"), mapped,);
	}

	#[test]
	fn map_backwards() {
		let mapper = KeyMap::<_From, _To>::default();
		let mapped = mapper.try_map_backwards(_To("small"));

		assert_eq!(Some(_From::Small), mapped);
	}
}
