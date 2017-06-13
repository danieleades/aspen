use node::{Node, Internals};
use status::Status;

/// A node whose status is determined by running a function on its child's status.
///
/// This node will tick its child and then run the supplied function on the
/// child's return status.
///
/// # State
///
/// **Initialized:** Depends on function.
///
/// **Running:** Depends on function.
///
/// **Succeeded:** Depends on function.
///
/// **Failed:** Depends on function.
///
/// # Children
///
/// Takes a single child which is ticked or reset every time the `Decorator` is
/// ticked or reset. The child may be ticked to completion multiple times
/// before the decorator is done.
///
/// # Examples
///
/// A decorator that inverts the return status of its child:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// fn invert(s: Status) -> Status
/// {
///     if s == Status::Succeeded { Status::Failed }
///     else if s == Status::Failed { Status::Succeeded }
///     else { s }
/// }
///
/// let child = AlwaysSucceed::new();
/// let mut node = Decorator::new(child, invert);
/// assert_eq!(node.tick(), Status::Failed);
/// ```
pub struct Decorator<'a>
{
	/// Function that is performed on the child's status.
	func: Box<Fn(Status) -> Status + 'a>,

	/// Child node.
	child: Node<'a>,
}
impl<'a> Decorator<'a>
{
	/// Creates a new Decorator node with the supplied child node and function
	/// to be run on the child's status.
	pub fn new<F>(child: Node<'a>, func: F) -> Node<'a>
		where F: Fn(Status) -> Status + 'a
	{
		let internals = Decorator { func: Box::new(func), child: child };
		Node::new(internals)
	}
}
impl<'a> Internals for Decorator<'a>
{
	fn tick(&mut self) -> Status
	{
		// If the child has already run, this shouldn't change results since it will
		// just return its last status
		let child_status = self.child.tick();
		(*self.func)(child_status)
	}

	fn reset(&mut self)
	{
		self.child.reset();
	}

	fn children(&self) -> Vec<&Node>
	{
		vec![&self.child]
	}

	/// Returns the string "Decorator".
	fn type_name(&self) -> &'static str
	{
		"Decorator"
	}
}

/// A node that returns the opposite completed status from its child.
///
/// This node inverts the compeleted status of its child node. If the child
/// succeeds, this node fails and vice versa. This node is considered running if
/// the child is running.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While the child node is running.
///
/// **Succeeded:** Once the child node fails.
///
/// **Failed:** Once the child node succeeds.
///
/// # Children
///
/// One node that will be ticked or reset whenever the parent is ticked or
/// reset.
///
/// # Examples
///
/// Causes a failed node to succeed:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = Invert::new(AlwaysFail::new());
/// assert_eq!(node.tick(), Status::Succeeded);
/// ```
pub struct Invert<'a>
{
	/// Child node.
	child: Node<'a>,
}
impl<'a> Invert<'a>
{
	/// Creates a new `Invert` node.
	pub fn new(child: Node<'a>) -> Node<'a>
	{
		Node::new(Invert { child: child })
	}
}
impl<'a> Internals for Invert<'a>
{
	fn tick(&mut self) -> Status
	{
		match self.child.tick() {
			Status::Succeeded => Status::Failed,
			Status::Failed    => Status::Succeeded,
			s @ _             => s,
		}
	}

	fn reset(&mut self)
	{
		// Reset the child
		self.child.reset();
	}

	fn children(&self) -> Vec<&Node>
	{
		vec![&self.child]
	}

	/// Returns the string "Invert".
	fn type_name(&self) -> &'static str
	{
		"Invert"
	}
}

#[cfg(test)]
mod test
{
	use status::Status;
	use std_nodes::*;

	fn rotate(s: Status) -> Status
	{
		match s {
			Status::Initialized => Status::Running,
			Status::Running => Status::Succeeded,
			Status::Succeeded => Status::Failed,
			Status::Failed => Status::Initialized,
		}
	}

	#[test]
	fn decorator()
	{
		// Test the first rotation
		let suc_child = YesTick::new(Status::Succeeded);
		let mut suc_dec = Decorator::new(suc_child, rotate);
		let suc_status = suc_dec.tick();
		drop(suc_dec);
		assert_eq!(suc_status, rotate(Status::Succeeded));

		// Test the second rotation
		let run_child = YesTick::new(Status::Running);
		let mut run_dec = Decorator::new(run_child, rotate);
		let run_status = run_dec.tick();
		drop(run_dec);
		assert_eq!(run_status, rotate(Status::Running));

		// Test the final rotation
		let fail_child = YesTick::new(Status::Failed);
		let mut fail_dec = Decorator::new(fail_child, rotate);
		let fail_status = fail_dec.tick();
		drop(fail_dec);
		assert_eq!(fail_status, rotate(Status::Failed));
	}

	#[test]
	fn invert_success_to_failure()
	{
		let mut s2f = Invert::new(YesTick::new(Status::Failed));
		let s2fs = s2f.tick();
		drop(s2f);
		assert_eq!(s2fs, Status::Succeeded);
	}

	#[test]
	fn invert_failure_to_success()
	{
		let mut f2s = Invert::new(YesTick::new(Status::Succeeded));
		let f2ss = f2s.tick();
		drop(f2s);
		assert_eq!(f2ss, Status::Failed);
	}

	#[test]
	fn invert_running_as_running()
	{
		let mut r = Invert::new(YesTick::new(Status::Running));
		let rs = r.tick();
		drop(r);
		assert_eq!(rs, Status::Running);
	}
}
