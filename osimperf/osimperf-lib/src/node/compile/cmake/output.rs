/// Returns size and compile time.
pub struct CMakeOutput {
    pub size: usize,
    pub duration: Duration,
}

impl CMakeOutput {
    pub fn new(
        id: Id<'a>, // For getting the target folders.
        focus: Focus,
        duration: Duration,
    ) -> Result<Self> {
        todo!()
    }
}
