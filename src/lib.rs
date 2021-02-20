#[macro_use]
extern crate lazy_static;

mod front_of_house;

use front_of_house::hosting;

pub fn test() {
    println!("Test");

    hosting::add_to_waitlist();
}
