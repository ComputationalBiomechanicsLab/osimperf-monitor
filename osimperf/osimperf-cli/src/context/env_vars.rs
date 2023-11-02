use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::path::PathBuf;

use crate::{MODELS_ENV_VAR, SETUP_ENV_VAR, CONTEXT_ENV_VAR, OPENSIM_BUILD_ENV_VAR, OPENSIM_SRC_ENV_VAR, OPENSIM_INSTALL_ENV_VAR, INSTALL_ENV_VAR, EnvVar};

#[derive(Debug, Clone, Default)]
struct EnvVars {
    pub opensim_build: Option<PathBuf>,
    pub opensim_source: Option<PathBuf>,
    pub opensim_install: Option<PathBuf>,
    pub install: Option<PathBuf>,
    pub models: Option<PathBuf>,
    pub test_setup: Option<PathBuf>,
    pub test_context: Option<PathBuf>,
}

impl EnvVars {
    pub fn make(self) -> Vec<EnvVar> {
        let mut vars = Vec::new();
        if let Some(p) = self.models {
            vars.push(EnvVar::new(MODELS_ENV_VAR, &p))
        }
        if let Some(p) = self.test_setup {
            vars.push(EnvVar::new(SETUP_ENV_VAR, &p))
        }
        if let Some(p) = self.test_context {
            vars.push(EnvVar::new(CONTEXT_ENV_VAR, &p));

            let install = self
                .opensim_install
                .clone()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            // vars.push(EnvVar {
            //     key: "PATH".to_string(),
            //     value: format!("/bin:{}:{}/lib:{}/include", install, install, install),
            // });
            vars.push(EnvVar {
                key: "LD_LIBRARY_PATH".to_string(),
                value: format!("{}/opensim-core/lib", install),
            });
        }

        if let Some(p) = self.opensim_build {
            vars.push(EnvVar::new(OPENSIM_BUILD_ENV_VAR, &p))
        }
        if let Some(p) = self.opensim_source {
            vars.push(EnvVar::new(OPENSIM_SRC_ENV_VAR, &p))
        }
        if let Some(p) = self.opensim_install {
            vars.push(EnvVar::new(OPENSIM_INSTALL_ENV_VAR, &p))
        }
        if let Some(p) = self.install {
            vars.push(EnvVar::new(INSTALL_ENV_VAR, &p))
        }
        vars
    }
}
