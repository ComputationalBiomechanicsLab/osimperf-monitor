use crate::context::OPENSIM_BUILD_ENV_VAR;
use crate::context::OPENSIM_INSTALL_ENV_VAR;
use crate::context::OPENSIM_SRC_ENV_VAR;
use osimperf_lib::Command;

use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct CMakeCommands(pub Vec<(String, Command)>);

impl Default for CMakeCommands {
    fn default() -> CMakeCommands {
        let mut configure_dependecies_cmd = Command::new("cmake");
        configure_dependecies_cmd.add_arg("-B");
        configure_dependecies_cmd.add_arg(format!("${OPENSIM_BUILD_ENV_VAR}/dependencies"));
        configure_dependecies_cmd.add_arg("-S");
        configure_dependecies_cmd.add_arg(format!("${OPENSIM_SRC_ENV_VAR}/dependencies"));
        configure_dependecies_cmd.add_arg(format!(
            "-DCMAKE_INSTALL_PREFIX=${OPENSIM_INSTALL_ENV_VAR}/dependencies",
        ));
        configure_dependecies_cmd.add_arg("-DCMAKE_BUILD_TYPE=RelWithDebInfo");
        configure_dependecies_cmd.add_arg("-DOPENSIM_WITH_CASADI=OFF");
        configure_dependecies_cmd.add_arg("-DOPENSIM_WITH_TROPTER=OFF");

        let mut build_dependecies_cmd = Command::new("cmake");
        build_dependecies_cmd.add_arg("--build");
        build_dependecies_cmd.add_arg(format!("${OPENSIM_BUILD_ENV_VAR}/dependencies"));
        build_dependecies_cmd.add_arg("j14");

        let mut configure_opensim_cmd = Command::new("cmake");
        configure_opensim_cmd.add_arg("-B");
        configure_opensim_cmd.add_arg(format!("${OPENSIM_BUILD_ENV_VAR}/opensim-core"));
        configure_opensim_cmd.add_arg("-S");
        configure_opensim_cmd.add_arg(format!("${OPENSIM_SRC_ENV_VAR}"));
        configure_opensim_cmd.add_arg(format!(
            "-DCMAKE_INSTALL_PREFIX=${OPENSIM_INSTALL_ENV_VAR}/opensim-core",
        ));
        configure_opensim_cmd.add_arg(format!(
            "-DCMAKE_PREFIX_PATH=${OPENSIM_INSTALL_ENV_VAR}/dependencies"
        ));
        configure_opensim_cmd.add_arg("-DCMAKE_BUILD_TYPE=RelWithDebInfo");
        configure_opensim_cmd.add_arg("-DOPENSIM_BUILD_INDIVIDUAL_APPS=OFF");
        configure_opensim_cmd.add_arg("-DOPENSIM_INSTALL_UNIX_FHS=ON");
        configure_opensim_cmd.add_arg("-DBUILD_API_ONLY=OFF");
        configure_opensim_cmd.add_arg("-DBUILD_API_EXAMPLES=OFF");
        configure_opensim_cmd.add_arg("-DBUILD_JAVA_WRAPPING=OFF");
        configure_opensim_cmd.add_arg("-DBUILD_PYTHON_WRAPPING=OFF");
        configure_opensim_cmd.add_arg("-DBUILD_TESTING=OFF");
        configure_opensim_cmd.add_arg("-DOPENSIM_DOXYGEN_USE_MATHJAX=OFF");
        configure_opensim_cmd.add_arg("-DOPENSIM_WITH_CASADI=OFF");
        configure_opensim_cmd.add_arg("-DOPENSIM_WITH_TROPTER=OFF");
        configure_opensim_cmd
            .add_arg("-DOPENSIM_DEPENDENCIES_DIR=${OPENSIM_INSTALL_ENV_VAR}/dependencies");

        let mut build_opensim_cmd = Command::new("cmake");
        build_opensim_cmd.add_arg("--build");
        build_opensim_cmd.add_arg(format!("${OPENSIM_BUILD_ENV_VAR}/opensim-core"));
        build_opensim_cmd.add_arg("--target");
        build_opensim_cmd.add_arg("install");
        build_opensim_cmd.add_arg("j14");

        CMakeCommands(vec![
            (
                String::from("Configure dependencies"),
                configure_dependecies_cmd,
            ),
            (String::from("Build dependencies"), build_dependecies_cmd),
            (
                String::from("Configure opensim-core"),
                configure_opensim_cmd,
            ),
            (String::from("Build opensim-core"), build_opensim_cmd),
        ])
    }
}
