use std::time::Instant;
use num_format::{Locale, ToFormattedString};

// type_name_of() seems to dereference automatically so it can't tell the difference between a basic value and a
// reference to that value.

#[macro_export]
macro_rules! types {
    ( $( $x:expr ),* ) => {
        {
            $(
                println!("{} is {}", stringify!($x), $x.type_name_of());
            )*
        }
    };
}

#[macro_export]
macro_rules! vals {
    ( $( $x:expr ),* ) => {
        {
            $(
                println!("{} = {:?}",
					stringify!($x),
					$x);
            )*
        }
    };
}

#[macro_export]
macro_rules! altvals {
    ( $( $x:expr ),* ) => {
        {
            $(
                println!("{} = {:#?}",
					stringify!($x),
					$x);
            )*
        }
    };
}

#[macro_export]
macro_rules! typedvals {
    ( $( $x:expr ),* ) => {
        {
            $(
                println!("{} = {:?}\n\ttype is {}",
					stringify!($x),
					$x,
					$x.type_name_of());
            )*
        }
    };
}

#[macro_export]
macro_rules! showrc {
    ( $( $x:expr ),* ) => {
        {
            $(
                println!("{} = {:?}\n\ttype is {}\n\tstrong count = {}\n\tweak count = {}",
					stringify!($x),
					$x,
					$x.type_name_of(),
					Rc::strong_count(&$x),
					Rc::weak_count(&$x));
            )*
        }
    };
}

pub fn format_indent(depth: usize, s: &str) -> String {
	format!("{}{}", "    ".repeat(depth), s)
}

pub fn print_indent(depth: usize, s: &str) {
	println!("{}", format_indent(depth, s));
}

pub fn print_elapsed<F>(display: bool, case_label: &str, step_label: &str, mut f: F)
	where F: FnMut() -> ()
{
	let start = Instant::now();
	f();
	print_elapsed_from_start(display, case_label, step_label, start);
}

pub fn print_elapsed_from_start(display: bool, case_label: &str, step_label: &str, start: Instant)
{
	if display {
		println!("\n{}: {} = {:?}", case_label, step_label, start.elapsed());
	}
}

pub fn format_count(val: usize) -> String {
    val.to_formatted_string(&Locale::en)
}

