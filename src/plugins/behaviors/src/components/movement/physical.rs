use common::traits::thread_safe::ThreadSafe;
use std::marker::PhantomData;

#[derive(PartialEq, Debug)]
pub struct Physical<TMotion>(PhantomData<TMotion>)
where
	TMotion: ThreadSafe;
