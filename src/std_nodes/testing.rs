//! Standard nodes used for debugging purposes
use std::sync::Arc;
use std::marker::PhantomData;
use std::ops::Drop;
use node::{Node, Internals};
use status::Status;

/// Implements a node that will panic upon being ticked
pub struct NoTick<T: Send + Sync + 'static>
{
	pd: PhantomData<T>,
}
impl<T: Send + Sync + 'static> NoTick<T>
{
	/// Construct a new NoTick node
	pub fn new() -> Node<T>
	{
		let internals = NoTick { pd: PhantomData };
		Node::new(internals)
	}
}
impl<T: Send + Sync + 'static> Internals<T> for NoTick<T>
{
	fn tick(&mut self, _: &Arc<T>) -> Status
	{
		panic!("This node should not have been ticked");
	}

	fn reset(&mut self)
	{
		// No-op
	}

	fn type_name() -> &'static str
	{
		"NoTick"
	}
}

/// Implements a node that will panic if it is dropped without being ticked
pub struct YesTick<T: Send + Sync + 'static>
{
	pd: PhantomData<T>,

	/// The status that this node should return
	status: Status,

	/// Whether or not this node has been ticked
	ticked: bool,
}
impl<T: Send + Sync + 'static> YesTick<T>
{
	/// Create a new YesTick that always has the given status
	pub fn new(status: Status) -> Node<T>
	{
		let internals = YesTick {
			pd: PhantomData,
			status: status,
			ticked: false,
		};
		Node::new(internals)
	}
}
impl<T: Send + Sync + 'static> Internals<T> for YesTick<T>
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

	fn type_name() -> &'static str
	{
		"YesTick"
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

	/// The status this node is to return
	status: Status,

	/// The number of times remaining for this node to be ticked
	count: u32,

	/// Whether or not the node can be ticked more than the given count
	exact: bool,

	/// Marker to show if this has been ticked at all (used for `self.tick()`)
	ticked: bool,
}
impl<T: Send + Sync + 'static> CountedTick<T>
{
	/// Creates a new CountedTick that always has the given status
	pub fn new(status: Status, count: u32, exact: bool) -> Node<T>
	{
		let internals = CountedTick {
			pd: PhantomData,
			status: status,
			count: count,
			exact: exact,
			ticked: false,
		};
		Node::new(internals)
	}
}
impl<T: Send + Sync + 'static> Internals<T> for CountedTick<T>
{
	fn tick(&mut self, _: &Arc<T>) -> Status
	{
		if self.exact && self.count == 0 {
			panic!("Node was ticked too many times");
		}

		self.ticked = true;

		self.count = self.count.saturating_sub(1);
		self.status
	}

	fn reset(&mut self)
	{
		self.ticked = false;
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
