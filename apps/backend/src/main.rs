use std::{thread, time::Duration};

fn main() {
    println!("Henlo!!!");

    loop {
        thread::sleep(Duration::from_secs(10));
    }
}