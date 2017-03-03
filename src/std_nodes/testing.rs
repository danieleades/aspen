//! Standard nodes used for debugging purposes
use std::sync::Arc;
use std::marker::PhantomData;
use std::ops::Drop;
use node::{Node, Iter, IdType};
use status::Status;

/// Implements a node that will panic upon being ticked
pub struct NoTick<T: Send + Sync + 'static>
{
	pd: PhantomData<T>,

	/// The UID for this node
	id: IdType,
}
impl<T: Send + Sync + 'static> NoTick<T>
{
	/// Construct a new NoTick node
	pub fn new() -> NoTick<T>
	{
		NoTick { pd: PhantomData, id: ::node::uid() }
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

	fn id(&self) -> IdType
	{
		self.id
	}


	#[cfg(feature = "messages")]
	fn as_message(&self) -> ::node_message::NodeMsg
	{
		::node_message::NodeMsg {
			id: self.id,
			num_children: 0,
			children: Vec::new(),
			status: self.status() as i32,
			type_name: "NoTick".to_string(),
		}
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

	/// The UID for this node
	id: IdType,
}
impl<T: Send + Sync + 'static> YesTick<T>
{
	/// Create a new YesTick that always has the given status
	pub fn new(status: Status) -> YesTick<T>
	{
		YesTick {
			pd: PhantomData,
			status: status,
			ticked: false,
			id: ::node::uid(),
		}
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

	fn id(&self) -> IdType
	{
		self.id
	}


	#[cfg(feature = "messages")]
	fn as_message(&self) -> ::node_message::NodeMsg
	{
		::node_message::NodeMsg {
			id: self.id,
			num_children: 0,
			children: Vec::new(),
			status: self.status() as i32,
			type_name: "YesTick".to_string(),
		}
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

	/// The UID of this node
	id: IdType,
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
			id: ::node::uid(),
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

	fn id(&self) -> IdType
	{
		self.id
	}


	#[cfg(feature = "messages")]
	fn as_message(&self) -> ::node_message::NodeMsg
	{
		::node_message::NodeMsg {
			id: self.id,
			num_children: 0,
			children: Vec::new(),
			status: self.status() as i32,
			type_name: "CountedTick".to_string(),
		}
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
