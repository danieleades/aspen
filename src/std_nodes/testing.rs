//! Standard nodes used for debugging purposes
use std::sync::Arc;
use std::marker::PhantomData;
use std::ops::Drop;
use node::Node;
use status::Status;

/// Implements a node that will panic upon being ticked
pub struct NoTick<T: Send + Sync + 'static>
{
	pd: PhantomData<T>,
}
impl<T: Send + Sync + 'static> NoTick<T>
{
	/// Construct a new NoTick node
	pub fn new() -> NoTick<T>
	{
		NoTick { pd: PhantomData }
	}
}
impl<T: Send + Sync + 'static> Node<T> for NoTick<T>
{
	fn tick(&mut self, _: &Arc<T>) -> Status
	{
		panic!("This node should not have been ticked");
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

/// Implements a node that will panic if it is dropped without being ticked
pub struct YesTick<T: Send + Sync + 'static>
{
	pd: PhantomData<T>,
	status: Status,
	ticked: bool,
}
impl<T: Send + Sync + 'static> YesTick<T>
{
	/// Create a new YesTick that always has the given status
	pub fn new(status: Status) -> YesTick<T>
	{
		YesTick { pd: PhantomData, status: status, ticked: false }
	}
}
impl<T: Send + Sync + 'static> Node<T> for YesTick<T>
{
	fn tick(&mut self, _: &Arc<T>) -> Status
	{
		self.ticked = true;
		self.status
	}

	fn reset(&mut self)
	{
		self.ticked = false;
	}

	fn status(&self) -> Status
	{
		self.status
	}
}
impl<T: Send + Sync + 'static> Drop for YesTick<T>
{
	fn drop(&mut self)
	{
		if !self.ticked {
			panic!("This node should have been ticked")
		}
	}
}
