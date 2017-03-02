//! Nodes that have a constant, well-defined behavior
use std::sync::Arc;
use std::marker::PhantomData;
use node::Node;
use status::Status;

/// Implements a node that always returns that it has failed
#[derive(PartialEq, Eq, Debug, Copy, Clone, Default)]
pub struct AlwaysFail<T: Send + Sync + 'static>
{
	pd: PhantomData<T>,
}
impl<T: Send + Sync + 'static> AlwaysFail<T>
{
	/// Construct a new AlwaysFail node
	pub fn new() -> AlwaysFail<T>
	{
		AlwaysFail { pd: PhantomData }
	}
}
impl<T: Send + Sync + 'static> Node<T> for AlwaysFail<T>
{
	fn tick(&mut self, _: &Arc<T>) -> Status
	{
		Status::Failed
	}

	fn reset(&mut self)
	{
		// No-op
	}

	fn status(&self) -> Status
	{
		Status::Failed
	}
}

/// Implements a node that always returns that it has succeeded
#[derive(PartialEq, Eq, Debug, Copy, Clone, Default)]
pub struct AlwaysSucceed<T: Send + Sync + 'static>
{
	pd: PhantomData<T>,
}
impl<T: Send + Sync + 'static> AlwaysSucceed<T>
{
	/// Construct a new AlwaysSucceed node
	pub fn new() -> AlwaysSucceed<T>
	{
		AlwaysSucceed { pd: PhantomData }
	}
}
impl<T: Send + Sync + 'static> Node<T> for AlwaysSucceed<T>
{
	fn tick(&mut self, _: &Arc<T>) -> Status
	{
		Status::Succeeded
	}

	fn reset(&mut self)
	{
		// No-op
	}

	fn status(&self) -> Status
	{
		Status::Succeeded
	}
}

/// Implements a node that always returns that it is currently running
#[derive(PartialEq, Eq, Debug, Copy, Clone, Default)]
pub struct AlwaysRunning<T: Send + Sync + 'static>
{
	pd: PhantomData<T>,
}
impl<T: Send + Sync + 'static> AlwaysRunning<T>
{
	/// Construct a new AlwaysRunning node
	pub fn new() -> AlwaysRunning<T>
	{
		AlwaysRunning { pd: PhantomData }
	}
}
impl<T: Send + Sync + 'static> Node<T> for AlwaysRunning<T>
{
	fn tick(&mut self, _: &Arc<T>) -> Status
	{
		Status::Running
	}

	fn reset(&mut self)
	{
		// No-op
	}

	fn status(&self) -> Status
	{
		Status::Running
	}
}