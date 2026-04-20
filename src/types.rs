use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Finding {
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeleteStats {
    pub recovered_bytes: u64,
    pub errors: u64,
}

