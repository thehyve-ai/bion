use clap::Subcommand;
use opt_in::OptInCommand;
use opt_out::OptOutCommand;
use pause_key::PauseKeyCommand;
use register_key::RegisterKeyCommand;
use remove_key::RemoveKeyCommand;
use stats::StatsCommand;
use unpause_key::UnpauseKeyCommand;

mod onboard;
mod opt_in;
mod opt_out;
mod pause_key;
mod register_key;
mod remove_key;
mod stats;
mod unpause_key;

#[derive(Debug, Subcommand)]
pub enum NetworkCommands {
    // #[command(name = "onboard")]
    // Onboard(OnboardCommand),
    #[command(name = "opt-in")]
    OptIn(OptInCommand),

    #[command(name = "opt-out")]
    OptOut(OptOutCommand),

    #[command(name = "pause-key")]
    PauseKey(PauseKeyCommand),

    #[command(name = "register-key")]
    RegisterKey(RegisterKeyCommand),

    #[command(name = "remove-key")]
    RemoveKey(RemoveKeyCommand),

    #[command(name = "stats")]
    Stats(StatsCommand),

    #[command(name = "unpause-key")]
    UnpauseKey(UnpauseKeyCommand),
}
