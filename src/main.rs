use arguments::{Action, ListenEvent};
use clap::Parser;

mod arguments;
mod output;
mod spotify;

use spotify::{SongType, SpotifyConnector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = arguments::CliArgs::parse();

    let spotify = SpotifyConnector::new().await?;
    match args.action {
        Action::Next => match spotify.next_song().await {
            Ok(SongType::Song(song)) => {
                println!(
                    "Now Playing \"{}\" by {}",
                    song.title,
                    song.artists.join(",")
                );
            }

            Ok(SongType::Ad) => {
                println!("Advertisement");
            }

            Err(zbus::Error::MethodError(..)) => {
                println!("Spotify not running");
            }

            Err(e) => panic!("{:?}", e),
        },

        Action::Previous => match spotify.previous_song().await {
            Ok(SongType::Song(song)) => {
                if args.json {
                    println!("{}", serde_json::to_string(&song)?);
                } else {
                    println!(
                        "Now Playing \"{}\" by {}",
                        song.title,
                        song.artists.join(",")
                    );
                }
            }

            Ok(SongType::Ad) => {
                if args.json {
                    println!("{}", serde_json::to_string(&SongType::Ad)?);
                } else {
                    println!("Advertisement");
                }
            }

            Err(zbus::Error::MethodError(..)) => {
                println!("Spotify not running");
            }

            Err(e) => panic!("{:?}", e),
        },

        Action::Song => match spotify.get_song().await {
            Ok(SongType::Song(song)) => {
                if args.json {
                    println!("{}", serde_json::to_string(&song)?);
                } else {
                    println!(
                        "Current song is \"{}\" by {}",
                        song.title,
                        song.artists.join(",")
                    );
                }
            }

            Ok(SongType::Ad) => {
                if args.json {
                    println!("{}", serde_json::to_string(&SongType::Ad)?);
                } else {
                    println!("Advertisement");
                }
            }

            Err(zbus::Error::MethodError(..)) => {
                println!("Spotify not running");
            }

            Err(e) => panic!("{:?}", e),
        },

        Action::Status => {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string(&spotify::PlayingStatusMsg {
                        status: spotify.get_status().await?
                    })?
                );
            } else {
                println!("{}", spotify.get_status().await?.to_string());
            }
        }

        Action::Listen(ListenEvent::SongChanged) => loop {
            match spotify.get_song_changed().await {
                Ok(SongType::Song(song)) => {
                    if args.json {
                        println!("{}", serde_json::to_string(&song)?);
                    } else {
                        println!(
                            "Now Playing \"{}\" by {}",
                            song.title,
                            song.artists.join(",")
                        );
                    }
                }

                Ok(SongType::Ad) => {
                    if args.json {
                        println!("{}", serde_json::to_string(&SongType::Ad)?);
                    } else {
                        println!("Advertisement");
                    }
                }

                Err(zbus::Error::MethodError(..)) => {
                    panic!("Spotify not running");
                }

                Err(e) => panic!("{:?}", e),
            }
        },

        Action::Listen(ListenEvent::Toggled) => loop {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string(&spotify::PlayingStatusMsg {
                        status: spotify.get_status_changed().await?
                    })?
                );
            } else {
                println!("{}", spotify.get_status_changed().await?.to_string());
            }
        },

        Action::Play => spotify.play().await?,
        Action::Pause => spotify.pause().await?,
        Action::Toggle => spotify.toggle().await?,
    }

    Ok(())
}
