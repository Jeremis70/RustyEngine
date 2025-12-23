/// Audio group categories for organizational and volume control purposes.
///
/// Each sound can belong to a group, allowing for independent volume control
/// and easier management of different audio categories.
///
/// # Example
/// ```ignore
/// let sfx_volume = 0.8;
/// audio_system.set_group_volume(SoundGroup::Sfx, sfx_volume)?;
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SoundGroup {
    /// Master volume (affects all sounds)
    Master,
    /// Music tracks
    Music,
    /// Sound effects
    Sfx,
    /// UI sounds (buttons, notifications)
    Ui,
    /// Voice/dialogue
    Voice,
    /// Custom group with index 0-255
    Custom(u8),
}

impl SoundGroup {
    /// Convert group to a unique numeric identifier
    pub fn as_id(&self) -> u8 {
        match self {
            SoundGroup::Master => 0,
            SoundGroup::Music => 1,
            SoundGroup::Sfx => 2,
            SoundGroup::Ui => 3,
            SoundGroup::Voice => 4,
            SoundGroup::Custom(id) => *id,
        }
    }

    /// Get human-readable name for the group
    pub fn name(&self) -> &'static str {
        match self {
            SoundGroup::Master => "Master",
            SoundGroup::Music => "Music",
            SoundGroup::Sfx => "SFX",
            SoundGroup::Ui => "UI",
            SoundGroup::Voice => "Voice",
            SoundGroup::Custom(_) => "Custom",
        }
    }
}

impl std::fmt::Display for SoundGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
