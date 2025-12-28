use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: Option<bool>,
    pub fullscreen: Option<bool>,
    pub title: Option<String>,
    pub visible: Option<bool>,
    pub decorations: Option<bool>,
    pub maximized: Option<bool>,
    pub transparent: Option<bool>,
    pub continuous: Option<bool>,
    pub target_fps: Option<u32>,
    pub vsync: Option<bool>,

    /// Whether to grab/lock the cursor inside the window (FPS mouse look).
    pub cursor_grab: Option<bool>,
    /// Whether the cursor should be visible.
    pub cursor_visible: Option<bool>,

    /// Optional path to an image used as the window icon.
    /// Note: the current build enables PNG/JPEG/BMP decoding via the `image` crate.
    pub icon_path: Option<PathBuf>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: Some(800),
            height: Some(600),
            resizable: Some(true),
            fullscreen: Some(false),
            title: Some("RustyEngine".to_string()),
            visible: Some(true),
            decorations: Some(true),
            maximized: Some(false),
            transparent: Some(false),
            continuous: Some(false),
            target_fps: None,
            vsync: Some(false),

            cursor_grab: Some(false),
            cursor_visible: Some(true),

            icon_path: Some("assets/icons/rust-logo-256x256.png".into()),
        }
    }
}

impl WindowConfig {
    /// Validate width/height and target_fps if provided.
    pub fn validate(&self) -> Result<(), String> {
        if let (Some(w), Some(h)) = (self.width, self.height)
            && (w == 0 || h == 0)
        {
            return Err("Width and height must be > 0".into());
        }

        if let Some(fps) = self.target_fps
            && fps == 0
        {
            return Err("target_fps must be > 0".into());
        }

        Ok(())
    }

    pub fn builder() -> WindowConfigBuilder {
        WindowConfigBuilder {
            config: WindowConfig::default(),
        }
    }
}

pub struct WindowConfigBuilder {
    config: WindowConfig,
}

impl WindowConfigBuilder {
    pub fn width(mut self, w: u32) -> Self {
        self.config.width = Some(w);
        self
    }
    pub fn height(mut self, h: u32) -> Self {
        self.config.height = Some(h);
        self
    }
    pub fn resizable(mut self, v: bool) -> Self {
        self.config.resizable = Some(v);
        self
    }
    pub fn fullscreen(mut self, v: bool) -> Self {
        self.config.fullscreen = Some(v);
        self
    }
    pub fn title<S: Into<String>>(mut self, t: S) -> Self {
        self.config.title = Some(t.into());
        self
    }
    pub fn visible(mut self, v: bool) -> Self {
        self.config.visible = Some(v);
        self
    }
    pub fn decorations(mut self, v: bool) -> Self {
        self.config.decorations = Some(v);
        self
    }
    pub fn maximized(mut self, v: bool) -> Self {
        self.config.maximized = Some(v);
        self
    }
    pub fn transparent(mut self, v: bool) -> Self {
        self.config.transparent = Some(v);
        self
    }
    pub fn continuous(mut self, v: bool) -> Self {
        self.config.continuous = Some(v);
        self
    }
    pub fn target_fps(mut self, fps: u32) -> Self {
        self.config.target_fps = Some(fps);
        self
    }
    pub fn vsync(mut self, v: bool) -> Self {
        self.config.vsync = Some(v);
        self
    }

    pub fn cursor_grab(mut self, v: bool) -> Self {
        self.config.cursor_grab = Some(v);
        self
    }

    pub fn cursor_visible(mut self, v: bool) -> Self {
        self.config.cursor_visible = Some(v);
        self
    }

    pub fn icon_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.icon_path = Some(path.into());
        self
    }

    /// Disable the window icon (sets `icon_path` to None).
    pub fn no_icon(mut self) -> Self {
        self.config.icon_path = None;
        self
    }
    pub fn build(self) -> WindowConfig {
        self.config
    }
}
