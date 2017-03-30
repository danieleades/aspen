use std::time::{Instant, Duration};
use std::thread;
use std::fmt;

use node::Node;
use status::Status;

/// Main behavior tree struct.
///
/// A behavior tree is considered to have been run to completion when it
/// returns either `Status::Succeeded` or `Status::Failed` when ticked. Unlike a
/// `Node`, the `BehaviorTree` will not automatically reset itself when ticked.
/// Instead, ticking or running a completed behavior tree will just return the
/// value of the last tick - it must be explicitly reset.
pub struct BehaviorTree
{
	/// Root node of the behavior tree.
	root: Node
}
impl BehaviorTree
{
	/// Create a new behavior tree with the supplied `Node` as the root.
	pub fn new(root: Node) -> BehaviorTree
	{
		BehaviorTree { root: root }
	}
}
impl BehaviorTree
{
	/// Returns a reference to the root node.
	pub fn root(&self) -> &Node
	{
		&self.root
	}

	/// Tick the behavior tree a single time.
	///
	/// If the tree has already been run to completion, this will simply return
	/// the value of the last tick.
	pub fn tick(&mut self) -> Status
	{
		// If we're already done, just return the root status without ticking
		if self.root.status().is_done() {
			self.root.status()
		} else { self.root.tick() }
	}

	/// Reset the tree so that it can be run again.
	pub fn reset(&mut self)
	{
		self.root.reset()
	}

	/// Run the behavior tree until it either succeeds or fails.
	///
	/// This makes no guarantees that it will run at the specified frequency. If a single
	/// tick takes longer than the alloted tick time, then it will do so silently.
	///
	/// If the hook is supplied, it will be run after every tick. A reference to this
	/// behavior tree will be supplied as an argument.
	///
	/// NOTE: The only time this will return `Status::Running` is if the frequency is zero
	/// and the behavior tree is running after the first tick.
	pub fn run<F>(&mut self, freq: f32, mut hook: Option<F>) -> Status
		where F: FnMut(&BehaviorTree)
	{
		// Deal with the "special" case of a zero frequency
		if freq == 0.0f32 {
			let status = self.tick();
			if let Some(ref mut f) = hook {
				f(self);
			}

			return status;
		}

		// Figure out the time-per-cycle
		let cycle_dur_float = freq.recip();
		let cycle_dur = Duration::new(cycle_dur_float as u64,
		                              (cycle_dur_float.fract() * 1000000000.0f32) as u32);

		// Now, run at the given frequency
		let mut status = Status::Running;
		while status == Status::Running {
			let now = Instant::now();

			status = self.tick();
			if let Some(ref mut f) = hook {
				f(self);
			}

			let elapsed = now.elapsed();

			// Sleep for the remaining amount of time
			if !status.is_done() && freq.is_finite() && elapsed < cycle_dur {
				// Really, the Duration would take care of this case. However, specifying a
				// frequency of infinity means running as fast a possible. In that case, I do
				// not want to give this thread an opportunity to sleep at all
				thread::sleep(cycle_dur - elapsed);
			}
		}

		return status;
	}
}
impl fmt::Display for BehaviorTree
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "BehaviorTree:( status = {:?}, root = {} )", self.root.status(), self.root)
	}
}
