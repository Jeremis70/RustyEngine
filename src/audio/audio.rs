use std::fmt;
use std::path::Path;
use std::time::Duration;

use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SoundId(u32);

impl SoundId {
    pub(crate) fn new(id: u32) -> Self {
        SoundId(id)
    }
}

impl fmt::Display for SoundId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("audio output device unavailable: {0}")]
    OutputUnavailable(#[from] rodio::StreamError),
    #[error("failed to resolve current directory: {0}")]
    CurrentDirectory(#[source] std::io::Error),
    #[error("failed to normalize audio file path {path:?}: {source}")]
    PathNormalization {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open audio file {path:?}: {source}")]
    FileOpen {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to decode audio file {path:?}: {source}")]
    Decode {
        path: std::path::PathBuf,
        #[source]
        source: rodio::decoder::DecoderError,
    },
    #[error("failed to clone audio file handle {path:?}: {source}")]
    FileClone {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("sound {0} has not been loaded")]
    SoundNotLoaded(SoundId),
    #[error("failed to create audio sink: {0}")]
    SinkCreation(#[source] rodio::PlayError),
    #[error("failed to read audio metadata {path:?}: {source}")]
    Metadata {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type AudioResult<T> = Result<T, AudioError>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LoadStrategy {
    Auto,
    Buffered,
    Streaming,
}

pub trait AudioBackend {
    fn load(&mut self, path: &Path, strategy: LoadStrategy) -> AudioResult<SoundId>;
    fn play(&mut self, sound: SoundId) -> AudioResult<()>;
    fn stop(&mut self, sound: SoundId) -> AudioResult<()>;
    fn pause(&mut self, sound: SoundId) -> AudioResult<()>;
    fn resume(&mut self, sound: SoundId) -> AudioResult<()>;
    fn set_volume(&mut self, sound: SoundId, volume: f32) -> AudioResult<()>;
    fn stop_all(&mut self);
    fn set_master_volume(&mut self, volume: f32);
    fn is_playing(&self, sound: SoundId) -> bool;
    fn duration(&self, sound: SoundId) -> Option<Duration>;
    fn unload(&mut self, sound: SoundId) -> AudioResult<()>;
    fn unload_all(&mut self);
}

pub struct AudioSystem {
    backend: Box<dyn AudioBackend>,
}

impl AudioSystem {
    pub fn new(backend: Box<dyn AudioBackend>) -> Self {
        Self { backend }
    }

    pub fn load<P>(&mut self, path: P) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.load_with_strategy(path, LoadStrategy::Auto)
    }

    pub fn load_with_strategy<P>(&mut self, path: P, strategy: LoadStrategy) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.backend.load(path.as_ref(), strategy)
    }

    pub fn load_buffered<P>(&mut self, path: P) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.load_with_strategy(path, LoadStrategy::Buffered)
    }

    pub fn load_streaming<P>(&mut self, path: P) -> AudioResult<SoundId>
    where
        P: AsRef<Path>,
    {
        self.load_with_strategy(path, LoadStrategy::Streaming)
    }

    pub fn play(&mut self, sound: SoundId) -> AudioResult<()> {
        self.backend.play(sound)
    }

    pub fn stop(&mut self, sound: SoundId) -> AudioResult<()> {
        self.backend.stop(sound)
    }

    pub fn pause(&mut self, sound: SoundId) -> AudioResult<()> {
        self.backend.pause(sound)
    }

    pub fn resume(&mut self, sound: SoundId) -> AudioResult<()> {
        self.backend.resume(sound)
    }

    pub fn set_volume(&mut self, sound: SoundId, volume: f32) -> AudioResult<()> {
        self.backend.set_volume(sound, volume)
    }

    pub fn stop_all(&mut self) {
        self.backend.stop_all()
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.backend.set_master_volume(volume)
    }

    pub fn is_playing(&self, sound: SoundId) -> bool {
        self.backend.is_playing(sound)
    }

    pub fn duration(&self, sound: SoundId) -> Option<Duration> {
        self.backend.duration(sound)
    }

    pub fn unload(&mut self, sound: SoundId) -> AudioResult<()> {
        self.backend.unload(sound)
    }

    pub fn unload_all(&mut self) {
        self.backend.unload_all()
    }
}
