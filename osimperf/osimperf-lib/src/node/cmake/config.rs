use super::super::Focus;
use serde::{Deserialize, Serialize};

pub static CMAKE_CONFIG_FILE: &str = ".osimperf-cmake.conf";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CMakeConfig {
    common: Vec<String>,
    opensim: Vec<String>,
    dependencies: Vec<String>,
    opensim_and_dependencies: Vec<String>,
    tests: Vec<String>,
    pub num_jobs: usize,
}

impl CMakeConfig {
    pub fn cmake_args(&self, focus: Focus) -> Vec<String> {
        match focus {
            Focus::OpenSimCore => self
                .common
                .iter()
                .chain(self.opensim_and_dependencies.iter())
                .chain(self.opensim.iter())
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            Focus::Dependencies => self
                .common
                .iter()
                .chain(self.opensim_and_dependencies.iter())
                .chain(self.dependencies.iter())
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            Focus::TestsSource => self
                .common
                .iter()
                .chain(self.tests.iter())
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        }
    }
}

impl Default for CMakeConfig {
    fn default() -> Self {
        Self {
            common: vec!["-DCMAKE_BUILD_TYPE=RelWithDebInfo".to_string()],
            opensim: vec![
                "-DOPENSIM_BUILD_INDIVIDUAL_APPS=OFF".to_string(),
                "-DOPENSIM_INSTALL_UNIX_FHS=ON".to_string(),
                "-DBUILD_API_ONLY=OFF".to_string(),
                "-DBUILD_API_EXAMPLES=OFF".to_string(),
                "-DBUILD_JAVA_WRAPPING=OFF".to_string(),
                "-DBUILD_PYTHON_WRAPPING=OFF".to_string(),
                "-DBUILD_TESTING=ON".to_string(),
            ],
            opensim_and_dependencies: vec![
                "-DOPENSIM_WITH_CASADI=OFF".to_string(),
                "-DOPENSIM_WITH_TROPTER=OFF".to_string(),
            ],
            num_jobs: 3,
            dependencies: vec![],
            tests: vec![],
        }
    }
}
