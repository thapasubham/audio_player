use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

pub fn play_audio() -> Result<(), Box<dyn Error>> {
    let audio_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fogweaver.mp3");
    let file = File::open(&audio_path)?;

    let metadata = file.metadata()?;
    println!("File size: {} bytes", metadata.len());
    println!("File is readable: {:?}", metadata);

    let handle = rodio::DeviceSinkBuilder::open_default_sink()?;
    let player = rodio::Player::connect_new(handle.mixer());
    let source = rodio::Decoder::try_from(file)?;
    player.append(source);
    player.sleep_until_end();

    Ok(())
}