//! Standard nodes used for debugging purposes.
use std::ops::Drop;
use node::{Node, Internals};
use status::Status;

/// Implements a node that will panic upon being ticked.
pub struct NoTick;
impl NoTick
{
	/// Construct a new `NoTick` node.
	pub fn new() -> Node
	{
		Node::new(NoTick { })
	}
}
impl Internals for NoTick
{
	fn tick(&mut self) -> Status
	{
		panic!("This node should not have been ticked");
	}

	fn reset(&mut self)
	{
		// No-op
	}

	/// Returns the string "NoTick".
	fn type_name(&self) -> &'static str
	{
		"NoTick"
	}
}

/// Implements a node that will panic if it is dropped without being ticked.
pub struct YesTick
{
	/// The status that this node should return.
	status: Status,

	/// Whether or not this node has been ticked.
	ticked: bool,
}
impl YesTick
{
	/// Create a new `YesTick` that always has the given status
	pub fn new(status: Status) -> Node
	{
		let internals = YesTick { status: status, ticked: false };
		Node::new(internals)
	}
}
impl Internals for YesTick
{
	fn tick(&mut self) -> Status
	{
		self.ticked = true;
		self.status
	}

	fn reset(&mut self)
	{
		self.ticked = false;
	}

	/// Returns the string "YesTick".
	fn type_name(&self) -> &'static str
	{
		"YesTick"
	}
}
impl Drop for YesTick
{
	fn drop(&mut self)
	{
		if !self.ticked {
			panic!("This node should have been ticked")
		}
	}
}

/// Implements a node that must be ticked a specific number of times.
///
/// The number of ticks persists over a reset, so this means that the node must
/// be ticked a specific number of times before being dropped instead of before
/// being reset.
pub struct CountedTick
{
	/// The status this node is to return.
	status: Status,

	/// The number of times remaining for this node to be ticked.
	count: u32,

	/// Whether or not the node can be ticked more than the given count.
	exact: bool,
}
impl CountedTick
{
	/// Creates a new `CountedTick` that always has the given status.
	pub fn new(status: Status, count: u32, exact: bool) -> Node
	{
		let internals = CountedTick {
			status: status,
			count: count,
			exact: exact,
		};
		Node::new(internals)
	}
}
impl Internals for CountedTick
{
	fn tick(&mut self) -> Status
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

	/// Returns the string "CountedTick".
	fn type_name(&self) -> &'static str
	{
		"CountedTick"
	}
}
impl Drop for CountedTick
{
	fn drop(&mut self)
	{
		if self.count != 0 {
			panic!("Node was not ticked enough times: {} remaining", self.count);
		}
	}
}
