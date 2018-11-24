//! Behavior tree nodes and internal node logic.

use crate::status::Status;
use std::fmt;

/// Represents a generic node.
///
/// The logic of the node is controlled by the supplied `Tickable` object.
/// Nodes are considered to have been run to completion when they return either
/// `Status::Succeeded` or `Status::Failed` when ticked. If they are ticked after
/// completion, they will be reset before the tick logic is executed.
///
/// This class is largely just a wrapper around an `Tickable` object. This is
/// to enforce some runtime behavior.
pub struct Node<'a, W> {
	/// The status from the last time this node was ticked.
	status: Status,

	/// The internal logic for this node.
	internals: Box<Tickable<W> + 'a>,

	/// The name for this node.
	///
	/// If present, it will be used instead of the type name.
	name: Option<String>,
}
impl<'a, W> Node<'a, W> {
	/// Creates a new `Node` with the given `Tickable`.
	///
	/// The internals are used to govern the tick logic of the node.
	pub fn new<I>(internals: I) -> Node<'a, W>
	where
		I: Tickable<W> + 'a,
	{
		Node {
			status: Status::Initialized,
			internals: Box::new(internals),
			name: None,
		}
	}

	/// Gets the current status of the node.
	///
	/// This value will match the return value of the last call to `Node::tick`.
	pub fn status(&self) -> Status {
		self.status
	}

	/// Returns the name of this node.
	///
	/// Unless this node was renamed via the `named` method, this will be the
	/// type name of the underlying `Tickable` object.
	pub fn name(&self) -> &str {
		if let Some(ref name) = self.name {
			name
		} else {
			(*self.internals).type_name()
		}
	}

	/// Sets the name for this particular node.
	pub fn named<T: Into<String>>(mut self, name: Option<T>) -> Node<'a, W> {
		// We consume the node and return it to fit better into the current
		// pattern of making trees. By using a reference, named nodes would not
		// be able to be made inline. This also makes the macros look much nicer.
		let new_name = name.map(|x| x.into());
		if let Some(ref s) = new_name {
			trace!("Renaming node from {} to {}", self.name(), s);
		} else {
			trace!("Removing name from {}", self.name());
		}
		self.name = new_name;
		self
	}
}

impl<'a, W> Tickable<W> for Node<'a, W> {
	/// Ticks the node a single time.
	///
	/// If the node is currently considered to have run to completion, this
	/// will call `Node::reset` on the node before calling the internal tick
	/// logic.
	fn tick(&mut self, world: &mut W) -> Status {
		// Reset the node if it's already completed
		if self.status.is_done() {
			self.reset();
		}

		// Tick the internals
		trace!("Ticking node {}", self.name());
		self.status = (*self.internals).tick(world);
		return self.status;
	}

	/// Resets the node.
	///
	/// This returns the node to a state that is identical to when it was first
	/// created. If the node state is still `Initialized`, then the internal
	/// reset method will not be called.
	fn reset(&mut self) {
		if self.status != Status::Initialized {
			trace!("Resetting node {} ({:?})", self.name(), self.status());
			self.status = Status::Initialized;
			(*self.internals).reset();
		}
	}

	/// Returns a vector containing references to all of this node's children.
	///
	/// This is likely the most unstable part of Aspen, use with caution.
	fn children(&self) -> Vec<&Node<W>> {
		(*self.internals).children()
	}

	/// Returns the type of the node as a string literal
	fn type_name(&self) -> &str {
		self.internals.type_name()
	}

	/// Returns a concrete Node.
	/// 
	/// (Node.into_node() does precisely nothing)
	fn into_node<'b>(self) -> Node<'b, W> where Self: Sized + 'b {
		self
	}
}

impl<'a, W> fmt::Display for Node<'a, W> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}:( status = {:?}", self.name(), self.status())?;
		for child in self.children() {
			write!(f, ", {}", child)?;
		}
		write!(f, " )")
	}
}

/// The internal logic of a node.
///
/// This is the object that controls the tick behavior of the `Node`, with
/// `Node` just being a wrapper to enforce some runtime behavior.
pub trait Tickable<W> {
	/// Ticks the internal state of the node a single time.
	///
	/// Node internals should not automatically reset themselves. If a node has
	/// been run to completion, the `Node` that holds this object will call
	/// `reset` before ticking the node.
	///
	/// In other words, the `Tickable` will only ever be ticked when the node
	/// state is either `Status::Running` or `Status::Initialized`.
	fn tick(&mut self, world: &mut W) -> Status;

	/// Resets the internal state of the node.
	///
	/// This sets the node to a state that is identical to a newly constructed
	/// node. Note that this could be called when the node is in any state.
	fn reset(&mut self);

	/// Returns a vector of references to this node's children.
	///
	/// Default behavior is to return `None`, which should be suitable for any
	/// leaf node.
	///
	/// This is likely the most unstable part of Aspen, use with caution.
	fn children(&self) -> Vec<&Node<W>> {
		Vec::with_capacity(0)
	}

	/// Returns the type of the node as a string literal.
	///
	/// In general, this should be the name of the node type.
	/// The type_name should be fixed for all instances of a
	/// node type
	fn type_name(&self) -> &str;

	/// Consumes 'self' and returns a concrete Node struct.
	/// 
	/// This method is used to allow child nodes methods to
	/// accept any struct that implements Tickable.
	fn into_node<'b>(self) -> Node<'b, W> where Self: Sized + 'b {
		Node::new(self)
	}
}
