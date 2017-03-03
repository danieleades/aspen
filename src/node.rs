use std::sync::Arc;
use status::Status;

/// Type used for node UIDs
pub type IdType = u32;

/// Represents a node in the behavior tree
pub trait Node<T: Send + Sync + 'static>
{
	/// Ticks the node a single time.
	///
	/// NOTE: Nodes should not automatically reset themselves. This was chosen
	/// in order to remove the need for special "star" nodes. Having the nodes
	/// automatically reset can be simulated using a decorator node.
	fn tick(&mut self, world: &Arc<T>) -> Status;

	/// Resets the node.
	///
	/// This sets the node to a state that is identical to a newly constructed
	/// node.
	fn reset(&mut self);

	/// Gets the current status of the node.
	///
	/// This value will match the return value of the last call to `tick`
	fn status(&self) -> Status;

	/// Returns an `Iter` that will go over this node and all of its children
	fn iter(&self) -> Iter<T>;

	/// Returns the node's ID.
	///
	/// In theory, this should be unique but I do not know how to enforce that
	/// in Rust. The function `uid()` will always return a value that it hasn't
	/// returned before (within the limits of `u32`).
	fn id(&self) -> IdType;

	#[cfg(feature = "messages")]
	/// Create a new `NodeMsg` from this node
	fn as_message(&self) -> ::node_message::NodeMsg;
}

/// An iterator over a `Node<T>` and all of its children
pub struct Iter<'a, T: 'static> {
	me: Option<&'a Node<T>>,
	upcoming: Option<Vec<Iter<'a, T>>>,
}
impl<'a, T: 'static> Iter<'a, T>
{
	/// Creates a new `Iter<T>`
	pub fn new(me: &'a Node<T>, children: Option<Vec<Iter<'a, T>>>) -> Self
	{
		Iter { me: Some(me), upcoming: children }
	}
}
impl<'a, T: 'static> Iterator for Iter<'a, T>
{
	type Item = &'a Node<T>;

	fn next(&mut self) -> Option<Self::Item>
	{
		// First, check if we've iterated over our own node
		if self.me.is_some() {
			return self.me.take();
		}

		// If we haven't, try iterating over the children
		if let Some(ref mut v) = self.upcoming {
			// We have children, so try to get values from them in order
			for child_iter in v.iter_mut() {
				let next = child_iter.next();
				if next.is_some() {
					return next;
				}
			}
		}

		// Either no children or they're all exhausted
		None
	}
}

pub fn uid() -> IdType
{
	use std::sync::atomic::{AtomicUsize, Ordering};
	static COUNTER: AtomicUsize = AtomicUsize::new(0);

	COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[cfg(test)]
mod test
{
	use std::sync::atomic::AtomicBool;
	use super::Node;
	use status::Status;
	use std_nodes::*;

	#[test]
	fn iter_test()
	{
		let succeed = Box::new(AlwaysSucceed::new());
		let running = Box::new(AlwaysRunning::new());
		let fail = Box::new(AlwaysFail::new());

		let children: Vec<Box<Node<AtomicBool>>> = vec![succeed, running, fail];

		let root = Sequence::new(children);
		let mut iter = root.iter();

		// root
		assert_eq!(Status::Running, iter.next().unwrap().status());

		// succeed
		assert_eq!(Status::Succeeded, iter.next().unwrap().status());

		// running
		assert_eq!(Status::Running, iter.next().unwrap().status());

		// fail
		assert_eq!(Status::Failed, iter.next().unwrap().status());

		// fin
		assert!(iter.next().is_none());
	}
}
