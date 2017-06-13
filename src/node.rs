//! Behavior tree nodes and internal node logic.

use std::fmt;
use status::Status;

/// Represents a generic node
///
/// The logic of the node is controlled by the supplied `Internals` object.
/// Nodes are considered to have been run to completion when they return either
/// `Status::Succeeded` or `Status::Failed` when ticked. If they are ticked after
/// completion, they will be reset before the tick logic is executed.
///
/// This class is largely just a wrapper around an `Internals` object. This is
/// to enforce some runtime behavior.
pub struct Node<'a>
{
	/// The status from the last time this node was ticked
	status: Status,

	/// The internal logic for this node
	internals: Box<Internals + 'a>,
}
impl<'a> Node<'a>
{
	/// Creates a new `Node` with the given `Internals`.
	///
	/// The internals are used to govern the tick logic of the node.
	pub fn new<I>(internals: I) -> Node<'a>
		where I: Internals + 'a
	{
		Node {
			status: Status::Initialized,
			internals: Box::new(internals),
		}
	}

	/// Ticks the node a single time.
	///
	/// If the node is currently considered to have run to completion, this
	/// will call `Node::reset` on the node before calling the internal tick
	/// logic.
	pub fn tick(&mut self) -> Status
	{
		// Reset the node if it has already been completed
		if self.status.is_done() {
			self.reset();
		}

		// Tick the internals
		self.status = (*self.internals).tick();
		return self.status;
	}

	/// Resets the node.
	///
	/// This returns the node to a state that is identical to when it was first
	/// created.
	pub fn reset(&mut self)
	{
		self.status = Status::Initialized;
		(*self.internals).reset();
	}

	/// Gets the current status of the node.
	///
	/// This value will match the return value of the last call to `Node::tick`.
	pub fn status(&self) -> Status
	{
		self.status
	}

	/// Returns a vector containing references to all of this node's children.
	pub fn children(&self) -> Option<Vec<&Node>>
	{
		(*self.internals).children()
	}

	/// Returns the name of this node.
	///
	/// This will usually be the type of the node, e.g. "Sequence". There are
	/// plans to allow nodes to have unique names.
	pub fn name(&self) -> &'static str
	{
		(*self.internals).type_name()
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
			status: self.status.into(),
			name: self.name().to_string(),
		}
	}
}
impl<'a> fmt::Display for Node<'a>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{}:( status = {:?}", self.name(), self.status())?;
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
	/// Node internals should not automatically reset themselves. If a node has
	/// been run to completion, the `Node` that holds this object will call
	/// `reset` before ticking the node.
	///
	/// In other words, the `Internals` will only ever be ticked when the node
	/// state is either `Status::Running` or `Status::Initialized`.
	fn tick(&mut self) -> Status;

	/// Resets the internal state of the node.
	///
	/// This sets the node to a state that is identical to a newly constructed
	/// node. Note that this could be called when the node is in any state.
	fn reset(&mut self);

	/// Returns a vector of references to this node's children.
	///
	/// Default behavior is to return `None`, which should be suitable for any
	/// leaf node.
	fn children(&self) -> Option<Vec<&Node>>
	{
		None
	}

	/// Returns the type of the node as a string literal.
	///
	/// In general, this should be the name of the node type. However, there
	/// are plans to add a "name" property to the `Node` struct, at which point
	/// this function will be depricated.
	fn type_name(&self) -> &'static str;
}
