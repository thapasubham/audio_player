
mod player;

fn main() {
    player::play_audio().expect("Audio playback failed");
}
