use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;

#[derive(Debug, Default, Clone)]
pub struct TrackInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<u64>,
    pub year: Option<String>,
    pub cover: Option<Vec<u8>>,
}

fn read_metadata(song: PathBuf) -> Result<TrackInfo, Box<dyn Error>> {
    let audio_path = PathBuf::from(song);
    let tagged_file = Probe::open(&audio_path)?.read()?;
    let duration = tagged_file.properties().duration().as_secs();
    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

    Ok(TrackInfo {
        title: tag.and_then(|t| t.title().map(|s| s.to_string())),
        artist: tag.and_then(|t| t.artist().map(|s| s.to_string())),
        album: tag.and_then(|t| t.album().map(|s| s.to_string())),
        duration: Some(duration),
        year: tag.and_then(|t| t.date()).map(|d| d.year.to_string()),
        cover: tag
            .and_then(|t| t.pictures().first())
            .map(|pic| pic.data().to_vec()),
    })
}

pub fn play_audio(on_start: impl FnOnce(&TrackInfo)) -> Result<(), Box<dyn Error>> {
    let audio_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fogweaver.mp3");
    let file = File::open(&audio_path)?;
    let current_playing = read_metadata(audio_path)?;
    on_start(&current_playing);

    let handle = rodio::DeviceSinkBuilder::open_default_sink()?;
    let player = rodio::Player::connect_new(handle.mixer());
    let source = rodio::Decoder::try_from(file)?;
    player.append(source);
    player.sleep_until_end();

    Ok(())
}
