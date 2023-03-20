use std::fmt::Debug;
pub fn debug_log<T: Debug>(message: T) {
    println!("{message:#?}")
}

pub fn log<T: ToString>(message: T) {
    println!("{}", message.to_string())
}
