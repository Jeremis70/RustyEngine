// === BASIC TYPES ===

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Digits
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,

    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
    LSuper,
    RSuper,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,

    NumPad0,
    NumPad1,
    NumPad2,
    NumPad3,
    NumPad4,
    NumPad5,
    NumPad6,
    NumPad7,
    NumPad8,
    NumPad9,
    NumPadAdd,
    NumPadSubtract,
    NumPadMultiply,
    NumPadDivide,
    NumPadDecimal,
    NumPadEnter,
    NumPadEquals,

    CapsLock,
    NumLock,
    ScrollLock,
    Fn,
    FnLock,

    MediaPlayPause,
    MediaStop,
    MediaNextTrack,
    MediaPrevTrack,
    VolumeUp,
    VolumeDown,
    VolumeMute,
    MediaSelect,
    Eject,
    Power,
    Sleep,
    WakeUp,

    BrowserBack,
    BrowserForward,
    BrowserRefresh,
    BrowserStop,
    BrowserSearch,
    BrowserFavorites,
    BrowserHome,
    LaunchMail,
    LaunchApp1,
    LaunchApp2,

    PrintScreen,
    Pause,
    Unknown,

    // Punctuation and symbols
    Backquote,
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backslash,
    IntlBackslash,
    IntlRo,
    IntlYen,
    Semicolon,
    Quote,
    Comma,
    Period,
    Slash,

    // Editing / system
    Help,
    ContextMenu,

    // IME / locale-specific
    Convert,
    NonConvert,
    KanaMode,
    Lang1,
    Lang2,
    Lang3,
    Lang4,
    Lang5,

    // Numpad additional keys
    NumPadBackspace,
    NumPadClear,
    NumPadClearEntry,
    NumPadComma,
    NumPadHash,
    NumPadMemoryAdd,
    NumPadMemoryClear,
    NumPadMemoryRecall,
    NumPadMemoryStore,
    NumPadMemorySubtract,
    NumPadParenLeft,
    NumPadParenRight,
    NumPadStar,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool, // Windows/Command key
}

// === EVENT STRUCTS ===
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub position: Position,
}

#[derive(Debug, Clone, Copy)]
pub struct TouchpadPressureEvent {
    pub pressure: f32,
    pub stage: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct AxisMotionEvent {
    pub axis: u32,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseWheelDelta {
    Lines(f32),  // Scroll in lines (most mice)
    Pixels(f32), // Scroll in pixels (precise trackpads)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Clone, Copy)]
pub struct Touch {
    pub id: u64,
    pub phase: TouchPhase,
    pub position: Position,
    pub force: Option<f32>, // Pressure (0.0 - 1.0)
}

#[derive(Debug, Clone)]
pub struct ImeEvent {
    pub kind: ImeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImeKind {
    Enabled,
    Preedit {
        text: String,
        cursor: Option<(usize, usize)>,
    },
    Commit(String),
    Disabled,
}

#[derive(Debug, Clone, Copy)]
pub struct GestureEvent {
    pub phase: TouchPhase,
    pub delta: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct PanEvent {
    pub phase: TouchPhase,
    pub delta: Position,
}
