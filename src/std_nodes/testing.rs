//! Standard nodes used for debugging purposes
use std::sync::Arc;
use std::marker::PhantomData;
use std::ops::Drop;
use node::{Node, Iter};
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

	fn iter(&self) -> Iter<T>
	{
		Iter::new(self, None)
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

	fn iter(&self) -> Iter<T>
	{
		Iter::new(self, None)
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

/// Implements a node that must be ticked a specific number of times (including resets)
pub struct CountedTick<T: Send + Sync + 'static>
{
	pd: PhantomData<T>,
	status: Status,
	count: u32,
	exact: bool,
}
impl<T: Send + Sync + 'static> CountedTick<T>
{
	/// Creates a new CountedTick that always has the given status
	pub fn new(status: Status, count: u32, exact: bool) -> CountedTick<T>
	{
		CountedTick {
			pd: PhantomData,
			status: status,
			count: count,
			exact: exact,
		}
	}
}
impl<T: Send + Sync + 'static> Node<T> for CountedTick<T>
{
	fn tick(&mut self, _: &Arc<T>) -> Status
	{
		if self.exact && self.count == 0 {
			panic!("Node was ticked too many times");
		}

		self.count = self.count.saturating_sub(1);
		self.status
	}

	fn reset(&mut self)
	{
		// No-op
	}

	fn status(&self) -> Status
	{
		self.status
	}

	fn iter(&self) -> Iter<T>
	{
		Iter::new(self, None)
	}
}
impl<T: Send + Sync + 'static> Drop for CountedTick<T>
{
	fn drop(&mut self)
	{
		if self.count != 0 {
			panic!("Node was not ticked enough times: {} remaining", self.count);
		}
	}
}
