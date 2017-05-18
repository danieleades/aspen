extern crate aspen;

use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::atomic::{ATOMIC_USIZE_INIT, ATOMIC_BOOL_INIT};
use std::{thread, time};
use aspen::BehaviorTree;
use aspen::std_nodes::{Sequence, Condition, Action, ShortAction};

const X: usize = 5;
const Y: usize = 3;

// Have some variable serve as the world state
static ADD_RES:  AtomicUsize = ATOMIC_USIZE_INIT;
static SUB_RES:  AtomicUsize = ATOMIC_USIZE_INIT;
static SUB_USED: AtomicBool  = ATOMIC_BOOL_INIT;

// Display the tree after each tick
fn hook(tree: &BehaviorTree)
{
	println!("{}", tree);
}

// Entry point of the program
fn main()
{
	// Create the tree - sleep to simulate work
	let root = Sequence::new(vec![
		// Addition node
		Action::new(|| {
			thread::sleep(time::Duration::from_secs(1));
			ADD_RES.store(X + Y, Ordering::SeqCst);
			Ok(())
		}),

		// Condition node to check if we can safely do the subtraction
		Condition::new(|| X > Y ),

		// Subtraction node. Only runs if the condition is successful. This one
		// doesn't do a long task (there is not sleep statement), so we can use
		// a `ShortAction` node, which will not boot up a new thread.
		ShortAction::new(|| {
			SUB_RES.store(X - Y, Ordering::SeqCst);
			SUB_USED.store(true, Ordering::SeqCst);
			Ok(())
		})
	]);

	// Put it all in a tree, print it, and run it
	let mut tree = BehaviorTree::new(root);
	println!("{}", tree);
	let res = tree.run(4.0, Some(hook));

	println!("\nTree finished: {:?}", res);
	println!("\nX: {}\nY: {}", X, Y);
	println!("\nADD_RES: {}\nSUB_RES: {}\nSUB_USED: {}",
	         ADD_RES.load(Ordering::SeqCst),
	         SUB_RES.load(Ordering::SeqCst),
	         SUB_USED.load(Ordering::SeqCst));
}
