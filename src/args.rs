use clap::{ArgAction, Parser};
use log::LevelFilter;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, action(ArgAction::Count))]
    pub verbose: u8,
}

impl Args {
    pub fn log_level(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            3.. => LevelFilter::Trace,
        }
    }
}
