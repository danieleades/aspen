use std::sync::Arc;
use status::Status;

/// Type used for node UIDs
pub type IdType = i32;

/// Returns a new UID
fn uid() -> IdType
{
	use std::sync::atomic::{AtomicIsize, Ordering, ATOMIC_ISIZE_INIT};
	static COUNTER: AtomicIsize = ATOMIC_ISIZE_INIT;

	COUNTER.fetch_add(1, Ordering::SeqCst) as IdType
}


/// Represents a generic node
///
/// The logic of the node is controlled by the supplied `Internals` object
pub struct Node<T: Send + Sync + 'static>
{
	/// This node's UID
	id: IdType,

	/// The status from the last time this node was ticked
	status: Status,

	/// The internal logic for this node
	internals: Box<Internals<T>>,
}
impl<T: Send + Sync + 'static> Node<T>
{
	/// Creates a new `Node` with the given `Internals`
	pub fn new<I: Internals<T>>(internals: I) -> Node<T>
	{
		Node {
			id: uid(),
			status: Status::Initialized,
			internals: Box::new(internals),
		}
	}

	/// Ticks the node a single time
	pub fn tick(&mut self, world: &Arc<T>) -> Status
	{
		self.status = (*self.internals).tick(world);
		return self.status;
	}

	/// Resets the node
	pub fn reset(&mut self)
	{
		self.status = Status::Initialized;
		(*self.internals).reset();
	}

	/// Gets the current status of the node.
	///
	/// This value will match the return value of the last call to `tick`
	pub fn status(&self) -> Status
	{
		self.status
	}

	/// Returns this node's ID>
	///
	/// In theory, this should be universally unique. However, a UUID is too
	/// heavy for how this ID will be used, so it will only be unique within
	/// a given process.
	pub fn id(&self) -> IdType
	{
		self.id
	}

	#[cfg(feature = "messages")]
	/// Creates a new `NodeMsg` from this node
	pub fn as_message(&self) -> ::node_message::NodeMsg
	{
		let child_ids = (*self.internals).children_ids();

		::node_message::NodeMsg {
			id: self.id,
			num_children: child_ids.len() as i32,
			children: child_ids,
			status: self.status as i32,
			type_name: (*self.internals).type_name().to_string(),
		}
	}
}

/// The internal logic of a node
pub trait Internals<T: Send + Sync + 'static>
{
	/// Ticks the internal state of the node a single time.
	///
	/// NOTE: Nodes should not automatically reset themselves. This was chosen
	/// in order to remove the need for special "star" nodes. Having the nodes
	/// automatically reset can be simulated using a decorator node.
	fn tick(&mut self, world: &Arc<T>) -> Status;

	/// Resets the internal state of the node.
	///
	/// This sets the node to a state that is identical to a newly constructed
	/// node.
	fn reset(&mut self);

	/// Returns a vector of references to this node's children
	fn children(&self) -> Vec<&Node<T>>
	{
		Vec::new()
	}

	/// Returns a vector of this node's childrens' node IDs
	fn children_ids(&self) -> Vec<IdType>
	{
		Vec::new()
	}

	/// Returns the name of the node type as a string literal
	fn type_name() -> &'static str;
}
