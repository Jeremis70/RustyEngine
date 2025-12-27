use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::{OutputStream, OutputStreamBuilder, Source};

use crate::audio::{AudioBackend, AudioError, AudioResult, LoadStrategy, SoundGroup, SoundId};

struct StreamingAudio {
    path: PathBuf,
    file: Arc<File>,
}

enum AudioData {
    Buffered(Arc<rodio::source::Buffered<rodio::Decoder<BufReader<File>>>>),
    Streaming(StreamingAudio),
}

struct SoundEntry {
    data: AudioData,
    duration: Option<Duration>,
    volume: f32,
    pan: f32,          // -1.0 (left) to 1.0 (right), 0.0 is center
    pitch: f32,        // Playback speed multiplier
    group: SoundGroup, // Which group this sound belongs to
}

impl Default for SoundEntry {
    fn default() -> Self {
        panic!(
            "SoundEntry::default() should not be used directly, as AudioData::Buffered requires a valid source"
        );
    }
}

pub struct RodioBackend {
    sounds: HashMap<SoundId, SoundEntry>,
    active_sinks: Mutex<HashMap<SoundId, Vec<rodio::Sink>>>,
    next_id: u32,
    _stream: Option<OutputStream>,
    mixer: Arc<rodio::mixer::Mixer>,
    streaming_threshold: u64,
    master_volume: f32,
    group_volumes: HashMap<u8, f32>, // Group ID -> Volume
}

impl RodioBackend {
    const DEFAULT_STREAMING_THRESHOLD_BYTES: u64 = 8 * 1024 * 1024;

    pub fn new() -> AudioResult<Self> {
        Self::with_streaming_threshold(Self::DEFAULT_STREAMING_THRESHOLD_BYTES)
    }

    pub fn with_streaming_threshold(threshold: u64) -> AudioResult<Self> {
        // Try to create output stream
        let stream = OutputStreamBuilder::open_default_stream()?;
        let mixer = Arc::new(stream.mixer().clone());

        let mut group_volumes = HashMap::new();
        // Initialize all standard groups with full volume
        group_volumes.insert(SoundGroup::Master.as_id(), 1.0);
        group_volumes.insert(SoundGroup::Music.as_id(), 1.0);
        group_volumes.insert(SoundGroup::Sfx.as_id(), 1.0);
        group_volumes.insert(SoundGroup::Ui.as_id(), 1.0);
        group_volumes.insert(SoundGroup::Voice.as_id(), 1.0);

        Ok(Self {
            sounds: HashMap::new(),
            active_sinks: Mutex::new(HashMap::new()),
            next_id: 0,
            _stream: Some(stream),
            mixer,
            streaming_threshold: threshold,
            master_volume: 1.0,
            group_volumes,
        })
    }

    pub fn set_streaming_threshold(&mut self, threshold: u64) {
        self.streaming_threshold = threshold;
    }

    fn choose_strategy(&self, path: &Path, strategy: LoadStrategy) -> AudioResult<LoadStrategy> {
        match strategy {
            LoadStrategy::Auto => {
                let metadata = std::fs::metadata(path).map_err(|source| AudioError::Metadata {
                    path: path.to_path_buf(),
                    source,
                })?;
                if metadata.len() > self.streaming_threshold {
                    Ok(LoadStrategy::Streaming)
                } else {
                    Ok(LoadStrategy::Buffered)
                }
            }
            other => Ok(other),
        }
    }

    fn prune_finished_sinks(&self) {
        if let Ok(mut sinks_by_sound) = self.active_sinks.lock() {
            sinks_by_sound.retain(|_, sinks| {
                sinks.retain(|sink| !sink.empty());
                !sinks.is_empty()
            });
        }
    }
}

impl RodioBackend {
    fn clamp_volume(volume: f32) -> f32 {
        volume.clamp(0.0, 1.0)
    }

    fn clamp_pan(pan: f32) -> f32 {
        pan.clamp(-1.0, 1.0)
    }

    fn clamp_pitch(pitch: f32) -> f32 {
        pitch.clamp(0.5, 2.0)
    }

    /// Calculate effective volume considering sound volume, group volume, and master volume
    fn calculate_effective_volume(&self, sound: SoundId) -> f32 {
        if let Some(entry) = self.sounds.get(&sound) {
            let group_id = entry.group.as_id();
            let group_vol = self.group_volumes.get(&group_id).copied().unwrap_or(1.0);
            entry.volume * group_vol * self.master_volume
        } else {
            0.0
        }
    }

    /// Try to create a sink
    fn try_create_sink(&mut self) -> AudioResult<rodio::Sink> {
        // Try to create sink connected to current mixer
        let sink = rodio::Sink::connect_new(&self.mixer);
        Ok(sink)
    }

    fn apply_volume_to_sound_sinks(&self, sound: SoundId) {
        let effective_volume = self.calculate_effective_volume(sound);
        if let Ok(mut sinks_by_sound) = self.active_sinks.lock()
            && let Some(sinks) = sinks_by_sound.get_mut(&sound)
        {
            for sink in sinks.iter_mut() {
                sink.set_volume(effective_volume);
            }
        }
    }

    fn reapply_all_volumes(&self) {
        if let Ok(mut sinks_by_sound) = self.active_sinks.lock() {
            for sound_id in sinks_by_sound.keys().copied().collect::<Vec<_>>() {
                let effective_volume = self.calculate_effective_volume(sound_id);
                if let Some(sinks) = sinks_by_sound.get_mut(&sound_id) {
                    for sink in sinks.iter_mut() {
                        sink.set_volume(effective_volume);
                    }
                }
            }
        }
    }
}

impl AudioBackend for RodioBackend {
    fn load(&mut self, path: &Path, strategy: LoadStrategy) -> AudioResult<SoundId> {
        let path = path.to_path_buf();
        let effective_strategy = self.choose_strategy(&path, strategy)?;

        let id = SoundId::new();
        let entry = match effective_strategy {
            LoadStrategy::Auto => unreachable!("auto strategy must be resolved before loading"),
            LoadStrategy::Buffered => {
                let file = File::open(&path).map_err(|source| AudioError::FileOpen {
                    path: path.clone(),
                    source,
                })?;
                let reader = BufReader::new(file);
                let decoder = rodio::Decoder::new(reader).map_err(|source| AudioError::Decode {
                    path: path.clone(),
                    source,
                })?;
                let duration = decoder.total_duration();
                let source = decoder.buffered();
                SoundEntry {
                    data: AudioData::Buffered(Arc::new(source)),
                    duration,
                    volume: 1.0,
                    pan: 0.0,
                    pitch: 1.0,
                    group: SoundGroup::Sfx,
                }
            }
            LoadStrategy::Streaming => {
                let file = File::open(&path).map_err(|source| AudioError::FileOpen {
                    path: path.clone(),
                    source,
                })?;
                let verify = file.try_clone().map_err(|source| AudioError::FileClone {
                    path: path.clone(),
                    source,
                })?;
                let reader = BufReader::new(verify);
                let decoder = rodio::Decoder::new(reader).map_err(|source| AudioError::Decode {
                    path: path.clone(),
                    source,
                })?;
                let duration = decoder.total_duration();
                SoundEntry {
                    data: AudioData::Streaming(StreamingAudio {
                        path: path.clone(),
                        file: Arc::new(file),
                    }),
                    duration,
                    volume: 1.0,
                    pan: 0.0,
                    pitch: 1.0,
                    group: SoundGroup::Sfx,
                }
            }
        };

        self.next_id = self.next_id.wrapping_add(1);
        self.sounds.insert(id, entry);
        Ok(id)
    }

    fn play(&mut self, sound: SoundId) -> AudioResult<()> {
        self.prune_finished_sinks();

        let entry_volume = self
            .sounds
            .get(&sound)
            .ok_or(AudioError::SoundNotLoaded(sound))?
            .volume;

        // Try to create sink with device recovery if needed
        let sink = match self.try_create_sink() {
            Ok(sink) => sink,
            Err(e) => {
                log::error!("Failed to create audio sink: {}", e);
                return Err(e);
            }
        };
        sink.set_volume(entry_volume * self.master_volume);

        let entry = &self.sounds[&sound];
        match &entry.data {
            AudioData::Buffered(buffered) => {
                sink.append(buffered.as_ref().clone());
            }
            AudioData::Streaming(streaming) => {
                let file = streaming
                    .file
                    .try_clone()
                    .map_err(|source| AudioError::FileClone {
                        path: streaming.path.clone(),
                        source,
                    })?;
                let reader = BufReader::new(file);
                let decoder = rodio::Decoder::new(reader).map_err(|source| AudioError::Decode {
                    path: streaming.path.clone(),
                    source,
                })?;
                sink.append(decoder);
            }
        }

        sink.play();
        match self.active_sinks.lock() {
            Ok(mut sinks_by_sound) => {
                sinks_by_sound.entry(sound).or_default().push(sink);
            }
            Err(_) => {
                // Fallback: detach so playback is not interrupted if the lock is poisoned.
                sink.detach();
            }
        }
        Ok(())
    }

    fn stop(&mut self, sound: SoundId) -> AudioResult<()> {
        if !self.sounds.contains_key(&sound) {
            return Err(AudioError::SoundNotLoaded(sound));
        }
        self.prune_finished_sinks();
        if let Ok(mut sinks_by_sound) = self.active_sinks.lock()
            && let Some(mut sinks) = sinks_by_sound.remove(&sound)
        {
            for sink in sinks.iter_mut() {
                sink.stop();
            }
        }
        Ok(())
    }

    fn pause(&mut self, sound: SoundId) -> AudioResult<()> {
        if !self.sounds.contains_key(&sound) {
            return Err(AudioError::SoundNotLoaded(sound));
        }
        self.prune_finished_sinks();
        if let Ok(mut sinks_by_sound) = self.active_sinks.lock()
            && let Some(sinks) = sinks_by_sound.get_mut(&sound)
        {
            for sink in sinks.iter_mut() {
                sink.pause();
            }
        }
        Ok(())
    }

    fn resume(&mut self, sound: SoundId) -> AudioResult<()> {
        if !self.sounds.contains_key(&sound) {
            return Err(AudioError::SoundNotLoaded(sound));
        }
        self.prune_finished_sinks();
        if let Ok(mut sinks_by_sound) = self.active_sinks.lock()
            && let Some(sinks) = sinks_by_sound.get_mut(&sound)
        {
            for sink in sinks.iter_mut() {
                sink.play();
            }
        }
        Ok(())
    }

    fn set_volume(&mut self, sound: SoundId, volume: f32) -> AudioResult<()> {
        let clamped = Self::clamp_volume(volume);
        {
            let entry = self
                .sounds
                .get_mut(&sound)
                .ok_or(AudioError::SoundNotLoaded(sound))?;
            entry.volume = clamped;
        }
        self.apply_volume_to_sound_sinks(sound);
        Ok(())
    }

    fn stop_all(&mut self) {
        self.prune_finished_sinks();
        if let Ok(mut sinks_by_sound) = self.active_sinks.lock() {
            for sinks in sinks_by_sound.values_mut() {
                for sink in sinks.iter_mut() {
                    sink.stop();
                }
            }
            sinks_by_sound.clear();
        }
    }

    fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = Self::clamp_volume(volume);
        self.reapply_all_volumes();
    }

    fn is_playing(&self, sound: SoundId) -> bool {
        self.prune_finished_sinks();
        if let Ok(sinks_by_sound) = self.active_sinks.lock()
            && let Some(sinks) = sinks_by_sound.get(&sound)
        {
            return sinks.iter().any(|sink| !sink.empty());
        }
        false
    }

    fn duration(&self, sound: SoundId) -> Option<Duration> {
        self.sounds.get(&sound).and_then(|entry| entry.duration)
    }

    fn unload(&mut self, sound: SoundId) -> AudioResult<()> {
        if !self.sounds.contains_key(&sound) {
            return Err(AudioError::SoundNotLoaded(sound));
        }
        self.stop(sound)?;
        self.sounds.remove(&sound);
        Ok(())
    }

    fn unload_all(&mut self) {
        self.stop_all();
        self.sounds.clear();
    }

    fn set_pan(&mut self, sound: SoundId, pan: f32) -> AudioResult<()> {
        let clamped = Self::clamp_pan(pan);
        let entry = self
            .sounds
            .get_mut(&sound)
            .ok_or(AudioError::SoundNotLoaded(sound))?;
        entry.pan = clamped;
        // Note: Rodio doesn't support panning directly on sinks,
        // this would require a custom source that applies panning
        Ok(())
    }

    fn set_pitch(&mut self, sound: SoundId, pitch: f32) -> AudioResult<()> {
        let clamped = Self::clamp_pitch(pitch);
        let entry = self
            .sounds
            .get_mut(&sound)
            .ok_or(AudioError::SoundNotLoaded(sound))?;
        entry.pitch = clamped;
        // Note: Rodio doesn't support pitch shifting directly,
        // this would require a resampling source
        Ok(())
    }

    fn set_group(&mut self, sound: SoundId, group: SoundGroup) -> AudioResult<()> {
        let entry = self
            .sounds
            .get_mut(&sound)
            .ok_or(AudioError::SoundNotLoaded(sound))?;
        entry.group = group;
        // Reapply volume with new group
        self.apply_volume_to_sound_sinks(sound);
        Ok(())
    }

    fn set_group_volume(&mut self, group: SoundGroup, volume: f32) -> AudioResult<()> {
        let clamped = Self::clamp_volume(volume);
        let group_id = group.as_id();
        self.group_volumes.insert(group_id, clamped);
        // Reapply all volumes to affected sinks
        self.reapply_all_volumes();
        Ok(())
    }

    fn get_pan(&self, sound: SoundId) -> Option<f32> {
        self.sounds.get(&sound).map(|entry| entry.pan)
    }

    fn get_pitch(&self, sound: SoundId) -> Option<f32> {
        self.sounds.get(&sound).map(|entry| entry.pitch)
    }

    fn get_group(&self, sound: SoundId) -> Option<SoundGroup> {
        self.sounds.get(&sound).map(|entry| entry.group)
    }

    fn get_group_volume(&self, group: SoundGroup) -> Option<f32> {
        self.group_volumes.get(&group.as_id()).copied()
    }
}

// Custom Drop implementation to handle cleanup gracefully
impl Drop for RodioBackend {
    fn drop(&mut self) {
        // Clear active sinks before dropping the stream
        if let Ok(mut sinks) = self.active_sinks.lock() {
            sinks.clear();
        }
        // Take ownership of the stream and forget it to prevent error messages
        // during shutdown when the audio device may no longer be available
        if let Some(stream) = self._stream.take() {
            std::mem::forget(stream);
        }
        log::debug!("Audio backend shut down");
    }
}
