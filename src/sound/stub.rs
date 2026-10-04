/// No-op sound manager used when the `sound` feature is disabled.
pub struct SoundManager;

impl SoundManager {
    pub fn new() -> Option<Self> {
        None
    }

    pub fn play_sound(&mut self, _soundkit_id: u32) -> Option<u32> {
        None
    }

    pub fn play_sound_file(&mut self, _path: &str) -> Option<u32> {
        None
    }

    pub fn stop_sound(&mut self, _handle: u32) {}

    pub fn set_volume(&mut self, _handle: u32, _volume: f32) {}

    pub fn is_playing(&self, _handle: u32) -> bool {
        false
    }

    pub fn cleanup(&mut self) {}
}
