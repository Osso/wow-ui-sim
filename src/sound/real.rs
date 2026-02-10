use rodio::stream::{OutputStreamBuilder, OutputStream};
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
    sound_dir: PathBuf,
    /// SoundKit ID -> relative file path (e.g. 850 -> "igmainmenuopen.ogg").
    soundkit_map: HashMap<u32, &'static str>,
}

impl SoundManager {
    /// Initialize audio output. Returns `None` if no audio device is available.
    pub fn new(sound_dir: PathBuf) -> Option<Self> {
        let stream = OutputStreamBuilder::from_default_device()
            .ok()?
            .open_stream_or_fallback()
            .ok()?;
        Some(Self {
            stream,
            active_sounds: HashMap::new(),
            sound_dir,
            soundkit_map: build_soundkit_map(),
        })
    }

    /// Play a sound by SoundKit ID. Returns a handle on success.
    pub fn play_sound(&mut self, soundkit_id: u32) -> Option<u32> {
        let rel_path = self.soundkit_map.get(&soundkit_id)?;
        let full_path = self.sound_dir.join(rel_path);
        self.play_file(&full_path)
    }

    /// Play a sound file by path. Returns a handle on success.
    pub fn play_sound_file(&mut self, path: &str) -> Option<u32> {
        // Try as absolute path first, then relative to sound_dir
        let full_path = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.sound_dir.join(path)
        };
        self.play_file(&full_path)
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
}

/// Build the SoundKit ID -> filename mapping for common UI sounds.
fn build_soundkit_map() -> HashMap<u32, &'static str> {
    HashMap::from([
        (829, "igspellbookopen.ogg"),
        (830, "igspellbookclose.ogg"),
        (836, "igabilitypageturn.ogg"),
        (839, "igcharacterinfotab.ogg"),
        (841, "igcharacterinfoopen.ogg"),
        (850, "igmainmenuopen.ogg"),
        (851, "igmainmenuclose.ogg"),
        (856, "igmainmenuoption.ogg"),
        (857, "igmainmenuoptioncheckboxon.ogg"),
        (858, "igmainmenuoptioncheckboxoff.ogg"),
        // UI_CLASS_TALENT_OPEN/CLOSE_WINDOW — fallback to classic spellbook sounds
        (207757, "igspellbookopen.ogg"),
        (207758, "igspellbookclose.ogg"),
    ])
}
