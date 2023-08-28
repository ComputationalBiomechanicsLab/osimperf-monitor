#[derive(Debug)]
pub struct CompilationNodeHandle<'a> {
    focus: CurrentlyCompiling,
    handle: &'a mut CompilationNode,
}

impl<'a> CompilationNodeHandle<'a> {
    pub fn new(handle: &'a mut CompilationNode, focus: Focus) -> Result<Option<Self>> {
        // Try getting a lock on the file.

        // Set to: Compiling(zero%)
        todo!()
    }

    pub fn set_status(&mut self, status: Status) {
        todo!()
    }
}

pub struct Compiler {

}
