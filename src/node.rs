use std::fmt;
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
pub struct Node
{
	/// This node's UID
	id: IdType,

	/// The status from the last time this node was ticked
	status: Status,

	#[cfg(feature = "lcm")]
	/// Whether or not this node is a root node
	///
	/// This only affects outgoing LCM messages
	is_root: bool,

	/// The internal logic for this node
	internals: Box<Internals>,
}
impl Node
{
	#[cfg(not(feature = "lcm"))]
	/// Creates a new `Node` with the given `Internals`
	pub fn new<I: Internals + 'static>(internals: I) -> Node
	{
		Node {
			id: uid(),
			status: Status::Initialized,
			internals: Box::new(internals),
		}
	}

	#[cfg(feature = "lcm")]
	/// Creates a new `Node` with the given `Internals`
	pub fn new<I: Internals + 'static>(internals: I) -> Node
	{
		Node {
			id: uid(),
			status: Status::Initialized,
			is_root: false,
			internals: Box::new(internals),
		}
	}

	/// Ticks the node a single time
	pub fn tick(&mut self) -> Status
	{
		self.status = (*self.internals).tick();
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

	/// Returns a vector containing references to all of this node's children
	pub fn children(&self) -> Vec<&Node>
	{
		(*self.internals).children()
	}

	/// Returns a vector containing the IDs of all this node's children
	pub fn children_ids(&self) -> Vec<IdType>
	{
		(*self.internals).children_ids()
	}

	#[cfg(feature = "lcm")]
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
			is_root: self.is_root,
		}
	}
}
impl fmt::Display for Node
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{}:( id = {}, status = {:?}", (*self.internals).type_name(), self.id, self.status)?;
		for child in self.children() {
			write!(f, ", {}", child)?;
		}
		write!(f, " )")
	}
}
#[cfg(feature = "lcm")]
impl ::Rootable for Node
{
	fn set_root(&mut self, root: bool)
	{
		self.is_root = root;
	}
}

/// The internal logic of a node
pub trait Internals
{
	/// Ticks the internal state of the node a single time.
	///
	/// NOTE: Nodes should not automatically reset themselves. This was chosen
	/// in order to remove the need for special "star" nodes. Having the nodes
	/// automatically reset can be simulated using a decorator node.
	fn tick(&mut self) -> Status;

	/// Resets the internal state of the node.
	///
	/// This sets the node to a state that is identical to a newly constructed
	/// node.
	fn reset(&mut self);

	/// Returns a vector of references to this node's children
	fn children(&self) -> Vec<&Node>
	{
		Vec::new()
	}

	/// Returns a vector of this node's childrens' node IDs
	fn children_ids(&self) -> Vec<IdType>
	{
		Vec::new()
	}

	/// Returns the name of the node type as a string literal
	fn type_name(&self) -> &'static str;
}
