use serde::Serialize;
use tokio::time::Duration;
use zbus::{export::futures_util::StreamExt, Connection, Proxy};
use zvariant::{Array, Dict, Str, Value};

#[derive(Serialize, Debug, Default)]
pub enum SongType {
    #[default]
    Ad,

    Song(SongMetadata),
}

#[derive(Serialize)]
pub struct PlayingStatusMsg {
    pub status: PlayingStatus,
}

#[derive(Serialize)]
pub enum PlayingStatus {
    Playing,
    Paused,
}

impl From<String> for PlayingStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Playing" => Self::Playing,
            "Paused" => Self::Paused,
            &_ => unreachable!(),
        }
    }
}

impl ToString for PlayingStatus {
    fn to_string(&self) -> String {
        match self {
            PlayingStatus::Playing => "Playing".to_string(),
            PlayingStatus::Paused => "Paused".to_string(),
        }
    }
}

#[derive(Serialize, Debug, Default)]
pub struct SongMetadata {
    pub artists: Vec<String>,
    pub album: String,
    pub title: String,
}

impl TryFrom<Value<'_>> for SongType {
    type Error = zvariant::Error;

    fn try_from(value: Value<'_>) -> Result<Self, Self::Error> {
        let value = Dict::try_from(value)?;

        let artists: Vec<String> = value
            .get::<_, Array>("xesam:artist")?
            .ok_or_else(|| {
                zvariant::Error::Message("Could not find xesam:artist in Array".to_string())
            })?
            .get()
            .iter()
            .map(|v| Str::try_from(v.clone()).unwrap().as_str().to_owned())
            .collect();

        let album = value
            .get::<_, Str>("xesam:album")?
            .ok_or_else(|| zvariant::Error::Message("Could not find xesam:album".to_string()))?
            .as_str()
            .to_string();

        let title = value
            .get::<_, Str>("xesam:title")?
            .ok_or_else(|| zvariant::Error::Message("Could not find xesam:title".to_string()))?
            .as_str()
            .to_string();

        if artists.contains(&"Advertisement".to_string()) {
            Ok(Self::Ad)
        } else {
            Ok(Self::Song(SongMetadata {
                artists,
                album,
                title,
            }))
        }
    }
}

pub struct SpotifyConnector<'a> {
    _conn: Connection,
    proxy: Proxy<'a>,
}

impl<'a> SpotifyConnector<'a> {
    pub async fn new() -> Result<SpotifyConnector<'a>, zbus::Error> {
        let conn = Connection::session().await?;

        let proxy = Proxy::new(
            &conn,
            "org.mpris.MediaPlayer2.spotify",
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player",
        )
        .await?;

        Ok(Self { _conn: conn, proxy })
    }

    async fn poll_songs(&self, timeout: Duration) -> Result<Vec<SongType>, zbus::Error> {
        let mut songs: Vec<SongType> = Vec::new();
        let mut metadata_signal = self
            .proxy
            .receive_property_changed::<Value>("Metadata")
            .await;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout) => {
                    break;
                }

                Some(song) = metadata_signal.next() => {
                    songs.push(song.get().await?.try_into()?);
                }
            }
        }

        Ok(songs)
    }

    pub async fn next_song(&self) -> Result<SongType, zbus::Error> {
        self.proxy.call_noreply("Next", &()).await?;
        let mut songs = self.poll_songs(Duration::from_millis(100)).await?;
        Ok(songs.pop().unwrap())
    }

    pub async fn previous_song(&self) -> Result<SongType, zbus::Error> {
        self.proxy.call_noreply("Previous", &()).await?;
        let mut songs = self.poll_songs(Duration::from_millis(100)).await?;
        Ok(songs.pop().unwrap())
    }

    pub async fn get_song(&self) -> Result<SongType, zbus::Error> {
        let song: SongType = self
            .proxy
            .get_property::<Value>("Metadata")
            .await?
            .try_into()?;

        Ok(song)
    }

    pub async fn get_song_changed(&self) -> Result<SongType, zbus::Error> {
        let mut signal = self
            .proxy
            .receive_property_changed::<Value>("Metadata")
            .await;
        let song = signal.next().await.unwrap();

        Ok(song.get().await?.try_into()?)
    }

    pub async fn get_status_changed(&self) -> Result<PlayingStatus, zbus::Error> {
        Ok(PlayingStatus::from(
            self.proxy
                .receive_property_changed::<String>("PlaybackStatus")
                .await
                .next()
                .await
                .unwrap()
                .get()
                .await?,
        ))
    }

    pub async fn get_status(&self) -> Result<PlayingStatus, zbus::Error> {
        Ok(PlayingStatus::from(
            self.proxy.get_property::<String>("PlaybackStatus").await?,
        ))
    }

    pub async fn pause(&self) -> Result<(), zbus::Error> {
        self.proxy.call_noreply("Pause", &()).await
    }

    pub async fn play(&self) -> Result<(), zbus::Error> {
        self.proxy.call_noreply("Play", &()).await
    }

    pub async fn toggle(&self) -> Result<(), zbus::Error> {
        self.proxy.call_noreply("PlayPause", &()).await
    }
}
