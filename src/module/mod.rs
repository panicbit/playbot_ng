use self::prelude::*;

pub mod crate_info;
pub use self::crate_info::CrateInfo;

pub mod playground;
pub use self::playground::Playground;

// pub mod codedb;
pub mod egg;
pub use self::egg::Egg;

pub mod help;
pub use self::help::Help;

mod prelude {
    pub(in super) use crate::{
        Context,
        Flow,
        CommandRegistry,
    };
    pub use super::Module;
    pub use failure::Error;
}

pub trait Module {
    fn init(commands: &mut CommandRegistry) where Self: Sized;
}
