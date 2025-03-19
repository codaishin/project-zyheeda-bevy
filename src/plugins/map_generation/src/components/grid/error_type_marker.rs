use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub(super) struct TypeMarker<T> {
	_p: PhantomData<T>, // Never exposed, prevents building this struct
}
