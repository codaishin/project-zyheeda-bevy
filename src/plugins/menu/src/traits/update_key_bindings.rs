use common::traits::iterate::Iterate;

pub(crate) trait UpdateKeyBindings<TKey, TKeyCode> {
	fn update_key_bindings<TKeyMap>(&mut self, map: &TKeyMap)
	where
		for<'a> TKeyMap: Iterate<'a, TItem = (&'a TKey, &'a TKeyCode)>;
}
