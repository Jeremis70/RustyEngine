use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use rodio::Source;

use crate::audio::AudioBackend;
use crate::audio::SoundId;

pub struct RodioBackend {
    sounds: HashMap<
        SoundId,
        rodio::source::Buffered<rodio::Decoder<std::io::BufReader<std::fs::File>>>,
    >,
    next_id: u32,
    _stream: rodio::OutputStream,
    handle: rodio::OutputStreamHandle,
}

impl RodioBackend {
    pub fn new() -> Self {
        let (stream, handle) =
            rodio::OutputStream::try_default().expect("impossible d'initialiser la sortie audio");
        Self {
            sounds: HashMap::new(),
            next_id: 0,
            _stream: stream,
            handle,
        }
    }

    fn normalize_path(path: &str) -> PathBuf {
        Path::new(path)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(path))
    }
}

impl AudioBackend for RodioBackend {
    fn load(&mut self, path: &str) -> SoundId {
        let file = std::fs::File::open(path).expect("failed to open audio file");
        let reader = BufReader::new(file);
        let source = rodio::Decoder::new(reader)
            .expect("invalid audio file format")
            .buffered();

        let id = SoundId::new(self.next_id);
        self.next_id += 1;

        self.sounds.insert(id, source);
        id
    }

    fn play(&mut self, sound: SoundId) {
        if let Some(source) = self.sounds.get(&sound)
            && let Ok(sink) = rodio::Sink::try_new(&self.handle)
        {
            sink.append(source.clone());
            sink.play();
            sink.detach();
        }
    }
}
