#[derive(Copy, Clone, Debug)]
pub enum Focus {
    OpenCimCore,
    Dependencies,
    TestsSource,
}

impl Focus {
    pub fn to_str(&self) -> &str {
        match self {
            Self::OpenCimCore => "opensim-core",
            Self::Dependencies => "dependencies",
            Self::TestsSource => "tests",
        }
    }
}
