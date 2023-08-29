#[repr(usize)]
#[derive(Copy, Clone, Debug)]
pub enum Focus {
    Dependencies = 0,
    OpenCimCore = 1,
    TestsSource = 2,
}

impl Focus {
    pub fn to_str(&self) -> &str {
        match self {
            Self::OpenCimCore => "opensim-core",
            Self::Dependencies => "dependencies",
            Self::TestsSource => "tests",
        }
    }

    pub fn from(other: usize) -> Self {
        match other {
            0 => Self::Dependencies,
            1 => Self::OpenCimCore,
            2 => Self::TestsSource,
            _ => panic!(),
        }
    }
}
