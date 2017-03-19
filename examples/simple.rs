extern crate aspen;

use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::atomic::{ATOMIC_USIZE_INIT, ATOMIC_BOOL_INIT};
use aspen::BehaviorTree;
use aspen::std_nodes::{Sequence, Condition, Action};

const X: usize = 5;
const Y: usize = 3;

// Have some variable serve as the world state
static ADD_RES:  AtomicUsize = ATOMIC_USIZE_INIT;
static SUB_RES:  AtomicUsize = ATOMIC_USIZE_INIT;
static SUB_USED: AtomicBool  = ATOMIC_BOOL_INIT;

// Display the tree after each tick
fn hook(tree: &BehaviorTree)
{
	// TODO
}

// Entry point of the program
fn main()
{
	// Create all of our leaf nodes
	let add_node = Action::new(|| {
		ADD_RES.store(X + Y, Ordering::SeqCst);
		true
	});

	let sub_check = Condition::new(|| {
		X > Y
	});

	let sub_node = Action::new(|| {
		SUB_RES.store(X - Y, Ordering::SeqCst);
		SUB_USED.store(true, Ordering::SeqCst);
		true
	});

	// Then chain them together in a sequence node
	let children = vec![add_node, sub_check, sub_node];
	let root = Sequence::new(children);

	// Put it all in a tree and run it
	let mut tree = BehaviorTree::new(root);
	let res = tree.run(4.0, Some(hook));

	println!("\nTree finished: {:?}", res);
	println!("\nX: {}\nY: {}", X, Y);
	println!("\nADD_RES: {}\nSUB_RES: {}\nSUB_USED: {}",
	         ADD_RES.load(Ordering::SeqCst),
	         SUB_RES.load(Ordering::SeqCst),
	         SUB_USED.load(Ordering::SeqCst));
}
