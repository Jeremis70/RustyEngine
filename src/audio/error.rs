use std::path::PathBuf;
use thiserror::Error;

use super::sound::SoundId;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("audio output device unavailable: {0}")]
    OutputUnavailable(#[from] rodio::StreamError),
    #[error("failed to resolve current directory: {0}")]
    CurrentDirectory(#[source] std::io::Error),
    #[error("failed to normalize audio file path {path:?}: {source}")]
    PathNormalization {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open audio file {path:?}: {source}")]
    FileOpen {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to decode audio file {path:?}: {source}")]
    Decode {
        path: PathBuf,
        #[source]
        source: rodio::decoder::DecoderError,
    },
    #[error("failed to clone audio file handle {path:?}: {source}")]
    FileClone {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("sound {0:?} has not been loaded")]
    SoundNotLoaded(SoundId),
    #[error("failed to create audio sink: {0}")]
    SinkCreation(#[source] rodio::PlayError),
    #[error("failed to read audio metadata {path:?}: {source}")]
    Metadata {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid volume level: {0} (must be 0.0-1.0)")]
    InvalidVolume(f32),
    #[error("audio system not initialized")]
    NotInitialized,
    #[error("audio backend error: {0}")]
    Backend(String),
}

pub type AudioResult<T> = Result<T, AudioError>;
