pub struct InstallId<'a> {
    pub name: &'a str,
    pub branch: &'a str,
    pub hash: &'a str,
    pub date: &'a str,
}

impl<'a> InstallId<'a> {
    pub fn subfolder_name(&self) -> String {
        format!("{}-{}-{}-{}", self.name, self.branch, self.hash, self.date)
    }
}
