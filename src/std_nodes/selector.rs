//! Nodes that have children and tick them in a sequential order as long as they
//! fail.
//!
//! NOTE: There is no Selector* node, since the choice of not having the nodes
//! automatically reset causes a normal Selector node to have the same behavior
//! as a Selector*.
use std::sync::Arc;
use node::{Node, Iter, IdType};
use status::Status;

/// Implements a Selector node
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Success`. If none do, this node fails.
pub struct Selector<T: Send + Sync + 'static>
{
	/// Vector containing the children of this node
	children: Vec<Box<Node<T>>>,

	/// The UID of this node
	id: IdType,
}
impl<T: Send + Sync + 'static> Selector<T>
{
	/// Creates a new Selector node from a vector of Nodes
	pub fn new(children: Vec<Box<Node<T>>>) -> Selector<T>
	{
		Selector {
			children: children,
			id: ::node::uid(),
		}
	}
}
impl<T: Send + Sync + 'static> Node<T> for Selector<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// Tick the children in order
		for ptr in self.children.iter_mut() {
			let child_status = (*ptr).tick(world);
			if child_status != Status::Failed {
				return child_status;
			}
		}

		// All children failed
		Status::Failed
	}

	fn reset(&mut self)
	{
		// Reset all of our children
		for ptr in self.children.iter_mut() {
			(*ptr).reset();
		}
	}

	fn status(&self) -> Status
	{
		for ptr in self.children.iter() {
			let child_status = (*ptr).status();
			if child_status != Status::Failed {
				return child_status;
			}
		}

		// All children failed
		Status::Failed
	}

	fn iter(&self) -> Iter<T>
	{
		let kids: Vec<_> = self.children.iter().map(|x| (*x).iter()).collect();
		Iter::new(self, Some(kids))
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
			num_children: self.children.len() as i32,
			children: self.children.iter().map(|x| (*x).id()).collect(),
			status: self.status() as i32,
			type_name: "Selector".to_string(),
		}
	}
}

#[cfg(test)]
mod test
{
	use std::sync::Arc;
	use std::sync::atomic::AtomicBool;
	use node::Node;
	use status::Status;
	use std_nodes::*;

	#[test]
	fn check_running()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let failed = Box::new(YesTick::new(Status::Failed));
		let running = Box::new(YesTick::new(Status::Running));
		let err = Box::new(NoTick::new());

		// Put them all in a vector
		let mut children: Vec<Box<Node<AtomicBool>>> = Vec::new();
		children.push(failed);
		children.push(running);
		children.push(err);

		// Add them to a seluence node
		let mut sel = Selector::new(children);

		// Tick the seluence
		let status = sel.tick(&world);

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Running);
	}

	#[test]
	fn check_success()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let failed = Box::new(YesTick::new(Status::Failed));
		let success = Box::new(YesTick::new(Status::Succeeded));
		let err = Box::new(NoTick::new());

		// Put them all in a vector
		let mut children: Vec<Box<Node<AtomicBool>>> = Vec::new();
		children.push(failed);
		children.push(success);
		children.push(err);

		// Add them to a seluence node
		let mut sel = Selector::new(children);

		// Tick the seluence
		let status = sel.tick(&world);

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn check_fail()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let first = Box::new(YesTick::new(Status::Failed));
		let second = Box::new(YesTick::new(Status::Failed));

		// Put them all in a vector
		let mut children: Vec<Box<Node<AtomicBool>>> = Vec::new();
		children.push(first);
		children.push(second);

		// Add them to a seluence node
		let mut sel = Selector::new(children);

		// Tick the seluence
		let status = sel.tick(&world);

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Failed);
	}
}
