use super::view::ItemView;
use common::traits::iteration::IterFinite;

pub(crate) trait ViewEntityNames<TKey> {
	fn view_entity_names() -> Vec<&'static str>;
}

impl<TView, TKey> ViewEntityNames<TKey> for TView
where
	TView: ItemView<TKey>,
	TKey: IterFinite,
{
	fn view_entity_names() -> Vec<&'static str> {
		TKey::iterator()
			.map(|key| TView::view_entity_name(&key))
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::iteration::Iter;

	#[derive(Clone, Copy)]
	enum _Key {
		A,
		B,
	}

	impl IterFinite for _Key {
		fn iterator() -> Iter<Self> {
			Iter(Some(_Key::A))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_Key::A => Some(_Key::B),
				_Key::B => None,
			}
		}
	}

	struct _View;

	impl ItemView<_Key> for _View {
		type TFilter = ();
		type TViewComponents = ();

		fn view_entity_name(key: &_Key) -> &'static str {
			match key {
				_Key::A => "A",
				_Key::B => "B",
			}
		}
	}

	#[test]
	fn produce_entity_names() {
		assert_eq!(vec!["A", "B"], _View::view_entity_names());
	}
}
