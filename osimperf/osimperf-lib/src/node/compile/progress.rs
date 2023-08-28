
struct ProgressStreamer<'a> {
    pub status: &'a Status,
}

impl<'a> ProgressStreamer<'a> {
    pub fn new(status: &'a Status, name: impl ToString) {
        todo!()
    }

    pub fn take_status(self) -> &'a Status {
        todo!()
    }
}

impl<'a> Write for ProgressStreamer<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

