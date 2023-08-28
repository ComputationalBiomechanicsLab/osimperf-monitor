struct CMakeConfigurerer<'a, I, S> {
    pub source: PathBuf,
    pub build: PathBuf,
    pub install: PathBuf,
    pub cmake_args: I,
    pub raw_args: I,
    pub target: Option<S>,
}

impl CMakeConfigurerer {
    pub fn run(self) -> anyhow::Result<CMakeBuilder> {
        todo!()
    }
}

struct CMakeBuilder<'a, I, S> {
    pub build: PathBuf,
    pub target: Option<S>,
}

struct CmakeRunner<'a, I, S> {
}

impl<'a, I, S> CmakeRunner<'a, I, S>
where
    I: Iterator<Item = S>,
    S: ToString,
{

    pub fn new(
        id: Id<'a>, // For getting the target folders.
        source: Source<'a>, // Makes sure we checked out.
    ) -> Self {
        let repo = source.path()?;

    }

    fn new_opensim_core(
        node: &CompilationNode) -> Self {

    }

    pub fn start_compilation(&mut self, progress: &ProgressStreamer) -> anyhow::Result<()> {
        if self.install.exists() {
            erase_folder(&self.install)?;
        }
        let mut cmd_echo = Command::parse("echo running-cmake-command!");
        cmd_echo.run_and_stream(&mut self.progress);

        Ok(())
    }
}
