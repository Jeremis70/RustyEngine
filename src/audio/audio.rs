#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SoundId(u32);

impl SoundId {
    pub(crate) fn new(id: u32) -> Self {
        SoundId(id)
    }
}

pub trait AudioBackend {
    fn load(&mut self, path: &str) -> SoundId;
    fn play(&mut self, sound: SoundId);
}

pub struct AudioSystem {
    backend: Box<dyn AudioBackend>,
}

impl AudioSystem {
    pub fn new(backend: Box<dyn AudioBackend>) -> Self {
        Self { backend }
    }

    pub fn load(&mut self, path: &str) -> SoundId {
        self.backend.load(path)
    }

    pub fn play(&mut self, sound: SoundId) {
        self.backend.play(sound)
    }
}
