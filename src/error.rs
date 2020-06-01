#[derive(Debug)]
pub enum RedundantFileError {
    Io(std::io::Error),
    MismatchHash(u32, String, String),
    RecostructError(reed_solomon_erasure::Error),
    JSONError(serde_json::Error),
    RocksDBError(rocksdb::Error),
    NoDataFound,
}
