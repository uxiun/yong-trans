use std::{process::{Command, Stdio}, path::Path, ffi::OsStr};

use crate::util::print_error;

pub fn rg_sort_head<P>(
	path: P,
	head_n: usize,
) -> Option<String>
where
	P: AsRef<Path> + AsRef<OsStr>
{

	let mut rg = Command::new("rg")
		.arg("^[0-9]")
		.arg(&path)
		.stdout(Stdio::piped())
		.spawn()
		.unwrap();

	let mut sort = Command::new("sort")
		.arg("-n")
		.stdin(rg.stdout.unwrap())
		.stdout(Stdio::piped())
		.spawn()
		.unwrap();

	let mut head = Command::new("head")
		.arg("-n")
		.arg(head_n.to_string())
		.stdin(sort.stdout.unwrap())
		.stdout(Stdio::piped())
		.spawn()
		.unwrap();
	
	print_error(head.wait_with_output())
	.map(|output| print_error(
		String::from_utf8(output.stdout)
	))
	.flatten()

}