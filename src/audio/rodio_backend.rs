use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::Source;

use crate::audio::{AudioBackend, AudioError, AudioResult, LoadStrategy, SoundId};

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
}

pub struct RodioBackend {
    sounds: HashMap<SoundId, SoundEntry>,
    active_sinks: Mutex<HashMap<SoundId, Vec<rodio::Sink>>>,
    next_id: u32,
    _stream: rodio::OutputStream,
    handle: rodio::OutputStreamHandle,
    streaming_threshold: u64,
    master_volume: f32,
}

impl RodioBackend {
    const DEFAULT_STREAMING_THRESHOLD_BYTES: u64 = 8 * 1024 * 1024;

    pub fn new() -> AudioResult<Self> {
        Self::with_streaming_threshold(Self::DEFAULT_STREAMING_THRESHOLD_BYTES)
    }

    pub fn with_streaming_threshold(threshold: u64) -> AudioResult<Self> {
        let (stream, handle) = rodio::OutputStream::try_default()?;
        Ok(Self {
            sounds: HashMap::new(),
            active_sinks: Mutex::new(HashMap::new()),
            next_id: 0,
            _stream: stream,
            handle,
            streaming_threshold: threshold,
            master_volume: 1.0,
        })
    }

    pub fn set_streaming_threshold(&mut self, threshold: u64) {
        self.streaming_threshold = threshold;
    }

    fn normalize_path(path: &Path) -> AudioResult<PathBuf> {
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(AudioError::CurrentDirectory)?
                .join(path)
        };

        std::fs::canonicalize(&absolute_path).map_err(|source| AudioError::PathNormalization {
            path: absolute_path.clone(),
            source,
        })
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

    fn apply_volume_to_sound_sinks(&self, sound: SoundId) {
        if let Ok(mut sinks_by_sound) = self.active_sinks.lock()
            && let (Some(entry), Some(sinks)) =
                (self.sounds.get(&sound), sinks_by_sound.get_mut(&sound))
        {
            let target_volume = entry.volume * self.master_volume;
            for sink in sinks.iter_mut() {
                sink.set_volume(target_volume);
            }
        }
    }

    fn reapply_all_volumes(&self) {
        if let Ok(mut sinks_by_sound) = self.active_sinks.lock() {
            for (sound_id, sinks) in sinks_by_sound.iter_mut() {
                if let Some(entry) = self.sounds.get(sound_id) {
                    let target_volume = entry.volume * self.master_volume;
                    for sink in sinks.iter_mut() {
                        sink.set_volume(target_volume);
                    }
                }
            }
        }
    }
}

impl AudioBackend for RodioBackend {
    fn load(&mut self, path: &Path, strategy: LoadStrategy) -> AudioResult<SoundId> {
        let normalized_path = Self::normalize_path(path)?;
        let effective_strategy = self.choose_strategy(&normalized_path, strategy)?;

        let id = SoundId::new(self.next_id);
        let entry = match effective_strategy {
            LoadStrategy::Auto => unreachable!("auto strategy must be resolved before loading"),
            LoadStrategy::Buffered => {
                let file = File::open(&normalized_path).map_err(|source| AudioError::FileOpen {
                    path: normalized_path.clone(),
                    source,
                })?;
                let reader = BufReader::new(file);
                let decoder = rodio::Decoder::new(reader).map_err(|source| AudioError::Decode {
                    path: normalized_path.clone(),
                    source,
                })?;
                let duration = decoder.total_duration();
                let source = decoder.buffered();
                SoundEntry {
                    data: AudioData::Buffered(Arc::new(source)),
                    duration,
                    volume: 1.0,
                }
            }
            LoadStrategy::Streaming => {
                let file = File::open(&normalized_path).map_err(|source| AudioError::FileOpen {
                    path: normalized_path.clone(),
                    source,
                })?;
                let verify = file.try_clone().map_err(|source| AudioError::FileClone {
                    path: normalized_path.clone(),
                    source,
                })?;
                let reader = BufReader::new(verify);
                let decoder = rodio::Decoder::new(reader).map_err(|source| AudioError::Decode {
                    path: normalized_path.clone(),
                    source,
                })?;
                let duration = decoder.total_duration();
                SoundEntry {
                    data: AudioData::Streaming(StreamingAudio {
                        path: normalized_path.clone(),
                        file: Arc::new(file),
                    }),
                    duration,
                    volume: 1.0,
                }
            }
        };

        self.next_id = self.next_id.wrapping_add(1);
        self.sounds.insert(id, entry);
        Ok(id)
    }

    fn play(&mut self, sound: SoundId) -> AudioResult<()> {
        self.prune_finished_sinks();

        let entry = self
            .sounds
            .get(&sound)
            .ok_or(AudioError::SoundNotLoaded(sound))?;
        let sink = rodio::Sink::try_new(&self.handle).map_err(AudioError::SinkCreation)?;
        sink.set_volume(entry.volume * self.master_volume);

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
}
