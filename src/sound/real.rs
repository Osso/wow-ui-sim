use rodio::stream::{OutputStream, OutputStreamBuilder};
use rodio::{Decoder, Sink};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_HANDLE: AtomicU32 = AtomicU32::new(1);

/// Manages audio playback for the simulator.
pub struct SoundManager {
    stream: OutputStream,
    active_sounds: HashMap<u32, Sink>,
    cache_dir: PathBuf,
    /// SoundKit ID -> fileDataID from SoundKitEntry.db2.
    soundkit_map: HashMap<u32, u32>,
}

impl SoundManager {
    /// Initialize audio output. Returns `None` if no audio device is available.
    pub fn new() -> Option<Self> {
        let stream = OutputStreamBuilder::from_default_device()
            .ok()?
            .open_stream_or_fallback()
            .ok()?;
        let cache_dir = default_sound_cache_dir()?;
        Some(Self {
            stream,
            active_sounds: HashMap::new(),
            cache_dir,
            soundkit_map: build_soundkit_map(),
        })
    }

    /// Play a sound by SoundKit ID. Returns a handle on success.
    pub fn play_sound(&mut self, soundkit_id: u32) -> Option<u32> {
        let fdid = *self.soundkit_map.get(&soundkit_id)?;
        let path = self.ensure_fdid_cached(fdid)?;
        self.play_file(&path)
    }

    /// Play a sound file by path. Returns a handle on success.
    pub fn play_sound_file(&mut self, path: &str) -> Option<u32> {
        let full_path = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.ensure_path_cached(path)?
        };
        self.play_file(&full_path)
    }

    /// Scale one playing sound's volume (1.0 = unchanged).
    pub fn set_volume(&mut self, handle: u32, volume: f32) {
        if let Some(sink) = self.active_sounds.get(&handle) {
            sink.set_volume(volume);
        }
    }

    /// Stop a playing sound by handle.
    pub fn stop_sound(&mut self, handle: u32) {
        if let Some(sink) = self.active_sounds.remove(&handle) {
            sink.stop();
        }
    }

    /// Check if a sound handle is still playing.
    pub fn is_playing(&self, handle: u32) -> bool {
        self.active_sounds
            .get(&handle)
            .is_some_and(|sink| !sink.empty())
    }

    /// Remove finished sinks to free resources.
    pub fn cleanup(&mut self) {
        self.active_sounds.retain(|_, sink| !sink.empty());
    }

    fn play_file(&mut self, path: &Path) -> Option<u32> {
        let file = File::open(path).ok()?;
        let source = Decoder::new(BufReader::new(file)).ok()?;
        let sink = Sink::connect_new(self.stream.mixer());
        sink.append(source);
        let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        self.active_sounds.insert(handle, sink);
        Some(handle)
    }

    fn ensure_fdid_cached(&self, fdid: u32) -> Option<PathBuf> {
        let out_path = self.cache_dir.join(format!("{fdid}.ogg"));
        ensure_sound_cached(fdid, &out_path)
    }

    fn ensure_path_cached(&self, path: &str) -> Option<PathBuf> {
        let resolver = crate::asset_resolver_config::resolver();
        let fdid = resolver.lookup_path(path)?;
        self.ensure_fdid_cached(fdid)
    }
}

fn default_sound_cache_dir() -> Option<PathBuf> {
    let dir = dirs::cache_dir()?.join("wow-ui-sim/sounds");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn ensure_sound_cached(fdid: u32, out_path: &Path) -> Option<PathBuf> {
    if out_path.is_file() {
        return Some(out_path.to_path_buf());
    }
    let resolver = crate::asset_resolver_config::resolver();
    resolver.ensure_cached(fdid, out_path)
}

/// Build the SoundKit ID -> fileDataID mapping for common UI sounds.
fn build_soundkit_map() -> HashMap<u32, u32> {
    HashMap::from([
        (829, 567440),    // IG_SPELLBOOK_OPEN
        (830, 567496),    // IG_SPELLBOOK_CLOSE
        (836, 567472),    // IG_ABILITY_PAGE_TURN
        (839, 567507),    // IG_CHARACTER_INFO_TAB
        (841, 567422),    // IG_CHARACTER_INFO_OPEN
        (850, 567490),    // IG_MAINMENU_OPEN
        (851, 567464),    // IG_MAINMENU_CLOSE
        (856, 567407),    // IG_MAINMENU_OPTION
        (857, 567407),    // IG_MAINMENU_OPTION_CHECKBOX_ON
        (858, 567407),    // IG_MAINMENU_OPTION_CHECKBOX_OFF
        (207757, 567507), // UI_CLASS_TALENT_OPEN_WINDOW
        (207758, 567433), // UI_CLASS_TALENT_CLOSE_WINDOW
    ])
}

#[cfg(test)]
mod tests {
    use super::build_soundkit_map;

    #[test]
    fn soundkit_file_data_ids_are_in_limited_listfile() {
        let missing: Vec<_> = build_soundkit_map()
            .into_values()
            .filter(|fdid| !crate::limited_listfile::entries().any(|entry| entry.fdid == *fdid))
            .collect();

        assert!(
            missing.is_empty(),
            "SoundKit fileDataIDs missing from limited listfile: {missing:?}"
        );
    }
}
