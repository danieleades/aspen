//! Represents a node within a behavior tree

use std::fmt;
use status::Status;

/// Represents a generic node
///
/// The logic of the node is controlled by the supplied `Internals` object
pub struct Node
{
	/// The status from the last time this node was ticked
	status: Status,

	/// The internal logic for this node
	internals: Box<Internals>,
}
impl Node
{
	/// Creates a new `Node` with the given `Internals`
	pub fn new<I>(internals: I) -> Node
		where I: Internals + 'static
	{
		Node {
			status: Status::Initialized,
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

	/// Returns a vector containing references to all of this node's children.
	/// If this node is a leaf, this returns `None`
	pub fn children(&self) -> Option<&Vec<Node>>
	{
		(*self.internals).children()
	}

	#[cfg(feature = "lcm")]
	/// Creates a new `NodeMsg` from this node
	pub fn as_message(&self) -> ::node_message::NodeMsg
	{
		let kids = if let Some(kids) = self.children() {
			kids.iter().map(|c| c.as_message() ).collect()
		} else { Vec::new() };

		::node_message::NodeMsg {
			num_children: kids.len() as i32,
			children: kids,
			status: self.status as i32,
			type_name: (*self.internals).type_name().to_string(),
		}
	}
}
impl fmt::Display for Node
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{}:( id = {}, status = {:?}", (*self.internals).type_name(), self.id, self.status)?;
		if let Some(children) = self.children() {
			for child in children {
				write!(f, ", {}", child)?;
			}
		}
		write!(f, " )")
	}
}

/// The internal logic of a node.
///
/// This is the object that controls the tick behavior of the `Node`, with
/// `Node` just being a wrapper to enforce some runtime behavior.
pub trait Internals
{
	/// Ticks the internal state of the node a single time.
	///
	/// Nodes should not automatically reset themselves. This was chosen
	/// in order to remove the need for special "star" nodes. Having the nodes
	/// automatically reset can be simulated using a decorator node.
	fn tick(&mut self) -> Status;

	/// Resets the internal state of the node.
	///
	/// This sets the node to a state that is identical to a newly constructed
	/// node.
	fn reset(&mut self);

	/// Returns a vector of references to this node's children. Default
	/// behavior is to return `None`
	fn children(&self) -> Option<&Vec<Node>>
	{
		None
	}

	/// Returns the name of the node type as a string literal
	fn type_name(&self) -> &'static str;
}
