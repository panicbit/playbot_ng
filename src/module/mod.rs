use self::prelude::*;

pub(crate) mod crate_info;
pub(crate) use self::crate_info::CrateInfo;

pub(crate) mod playground;
pub(crate) use self::playground::Playground;

// pub mod codedb;
pub(crate) mod egg;
pub(crate) use self::egg::Egg;

pub(crate) mod help;
pub(crate) use self::help::Help;

mod prelude {
    pub(crate) use crate::{Context, CommandRegistry};
    pub(crate) use super::Module;
}

pub(crate) trait Module {
    fn init(commands: &mut CommandRegistry) where Self: Sized;
}
