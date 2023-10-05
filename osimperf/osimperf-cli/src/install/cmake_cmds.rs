use osimperf_lib::Command;
use serde::{Deserialize, Serialize};
use crate::context::OPENSIM_BUILD_ENV_VAR;
use crate::context::OPENSIM_SRC_ENV_VAR;
use crate::context::OPENSIM_INSTALL_ENV_VAR;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CMakeCommands(Vec<(String, Command)>);

impl Default for CMakeCommands {
    fn default() -> CMakeCommands {
        let mut configure_dependecies_cmd = Command::new("cmake");
        configure_dependecies_cmd.add_arg("-B");
        configure_dependecies_cmd.add_arg(format!("${OPENSIM_BUILD_ENV_VAR}/dependencies"));
        configure_dependecies_cmd.add_arg("-S");
        configure_dependecies_cmd.add_arg(format!("${OPENSIM_SRC_ENV_VAR}/dependencies"));
        configure_dependecies_cmd.add_arg(format!(
            "-DCMAKE_INSTALL_PREFIX=${}",
            OPENSIM_INSTALL_ENV_VAR
        ));

        let mut build_dependecies_cmd = Command::new("cmake");
        configure_dependecies_cmd.add_arg("--build");

        CMakeCommands(vec![
            (
                String::from("Configure dependencies"),
                configure_dependecies_cmd,
            ),
            (String::from("Build dependencies"), build_dependecies_cmd),
        ])
    }
}
