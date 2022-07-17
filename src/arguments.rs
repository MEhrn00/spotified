use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub action: Action,

    /// Output the information in JSON format
    #[clap(long)]
    pub json: bool,
}

#[derive(Debug, clap::Subcommand)]
pub enum Action {
    /// Listen for an event from spotify
    #[clap(subcommand)]
    Listen(ListenEvent),

    /// Play the next song
    Next,

    /// Play the previous song
    Previous,

    /// Pause the current song
    Pause,

    /// Play the current song
    Play,

    /// Toggle the play/pause button
    Toggle,

    /// Get the current play/pause status
    Status,

    /// Get the current song
    Song,
}

#[derive(Debug, clap::Subcommand)]
pub enum ListenEvent {
    /// Listen for when the song changes
    #[clap(name = "song")]
    SongChanged,

    /// Listen for when the play/pause button is toggled
    #[clap(name = "toggled")]
    Toggled,
}
