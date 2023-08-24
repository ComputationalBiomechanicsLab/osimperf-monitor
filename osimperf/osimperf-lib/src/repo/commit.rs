#[derive(Clone, Debug)]
pub struct Commit {
    pub hash: String,
    pub date: String,
    pub branch: String,
}

#[derive(Clone, Debug)]
pub struct Date {
    yyyy_mm_dd: String,
}

impl Date {
    pub fn new(yyyy_mm_dd: &str) -> Self {
        Self {
            yyyy_mm_dd: String::from(yyyy_mm_dd),
        }
    }
    pub fn to_str(&self) -> &str {
        &self.yyyy_mm_dd
    }
}

#[derive(Clone, Debug)]
pub struct Hash {
    value: String,
}

impl Hash {
    pub fn new(hash: &str) -> Self {
        todo!()
    }

    pub fn str(&self) -> &str {
        todo!()
    }

    pub fn short(&self) -> &str {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct Branch {
    value: String,
}

impl Branch {
    pub fn new(branch: String) {
        todo!()
    }

    pub fn get_str(&self) -> &str {
        todo!()
    }
}
