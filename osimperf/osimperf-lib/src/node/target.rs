#[repr(usize)]
#[derive(Copy, Clone, Debug)]
pub enum CompilationTarget {
    Dependencies = 0,
    OpenSimCore = 1,
    TestsSource = 2,
}

impl CompilationTarget {
    pub fn to_str(&self) -> &str {
        match self {
            Self::OpenSimCore => "opensim-core",
            Self::Dependencies => "dependencies",
            Self::TestsSource => "tests",
        }
    }

    pub fn short_desc(&self) -> &str {
        match self {
            Self::OpenSimCore => "osim",
            Self::Dependencies => "deps",
            Self::TestsSource => "src",
        }
    }

    pub fn from(other: usize) -> Self {
        match other {
            0 => Self::Dependencies,
            1 => Self::OpenSimCore,
            2 => Self::TestsSource,
            _ => panic!(),
        }
    }

    pub fn list_all() -> [Self; 3] {
        [Self::Dependencies, Self::OpenSimCore, Self::TestsSource]
    }
}
