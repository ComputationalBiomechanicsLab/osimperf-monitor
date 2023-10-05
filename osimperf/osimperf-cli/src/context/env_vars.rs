use super::Ctxt;

// OSimPerf environmental variables.
pub const OPENSIM_BUILD_ENV_VAR: &str = "OSIMPERF_OPENSIM_BUILD";
pub const OPENSIM_SRC_ENV_VAR: &str = "OSIMPERF_OPENSIM_SRC";
pub const OPENSIM_INSTALL_ENV_VAR: &str = "OSIMPERF_OPENSIM_INSTALL";

/// OSIMPERF_RESULTS
/// OSIMPERF_CONFIG

pub struct EnvVar {
    pub key: String,
    pub value: String,
}

impl EnvVar {
    pub fn opensim_build_dir(context: &Ctxt) -> Self {
        todo!()
    }

    pub fn opensim_install_dir(context: &Ctxt) -> Self {
        todo!()
    }

    pub fn results_dir(context: &Ctxt) -> Self {
        todo!()
    }

    pub fn config_dir(context: &Ctxt) -> Self {
        todo!()
    }
}
