// Can be created from the [Input]
pub struct Params {
    /// The commit we are checking out.
    pub hash: String,
    /// The date is for ordering results.
    pub date: String,
}

impl Params {
    pub fn new_last(
        input: &Input,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> anyhow::Result<Params> {
        todo!()
    }

    pub fn collect_daily_last(
        input: &Input,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> anyhow::Result<Vec<Params>> {
        todo!()
    }
}
