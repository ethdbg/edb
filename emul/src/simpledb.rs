//! a simple database that serializes to/from messagepack
use log::*;
use std::{
    io::prelude::*,
    io::SeekFrom,
    fs::{self, File, OpenOptions},
    marker::PhantomData,
    path::PathBuf,
    default::Default,
};
use flate2::{
    Compression,
    write::DeflateEncoder,
    read::DeflateDecoder,
};
use serde::{
    Serialize,
    de::DeserializeOwned
};
use super::err::{EmulError, StateError};

#[derive(Debug)]
crate struct SimpleDB<D: DeserializeOwned + Serialize + Default> {
    path: PathBuf,
    _marker: PhantomData<D>,
}
// TODO: Figure out a way to use MessagePack instead of JSON
// JSON is OK because we compress it
// compression bench: of ETH tipjar addr txs, block 0-6mil - uncompressed 100MB, compressed 3.9MB
/// A simple DB that allows saving/retrieving structures to/from a (compressed) file,
impl<D> SimpleDB<D> where D: DeserializeOwned + Serialize + Default {
    crate fn new(path: PathBuf) -> Result<Self, EmulError> {
        if !path.as_path().exists() {
            File::create(path.as_path()).map_err(|e| EmulError::State(StateError::Io(e)))?;
        }
        Ok(SimpleDB {
            path,
            _marker: PhantomData
        })
    }

    /// Save structure to a file, serializing to JSON and then compressing with DEFLATE
    crate fn save(&self, data: D) -> Result<(), EmulError> {
        self.mutate(|file| {
            let ser_data = serde_json::ser::to_vec(&data).map_err(|e| EmulError::State(StateError::Decoder(e)))?;
            let mut e = DeflateEncoder::new(file, Compression::default());
            e.write_all(ser_data.as_slice()).map_err(|e| EmulError::State(StateError::Io(e)))?;
            e.finish().map_err(|e| EmulError::State(StateError::Io(e)))?;
            Ok(())
        })?;
        Ok(())
    }

    /// Get structure from file, DEFLATING and then deserializing from JSON
    crate fn get(&self) -> Result<D, EmulError> {
        let meta = fs::metadata(self.path.as_path()).map_err(|e| EmulError::State(StateError::Io(e)))?;
        if meta.len() == 0 {
            info!("File length is 0");
            return Ok(D::default());
        }
        self.read(|file| {
            let mut deflater = DeflateDecoder::new(file);
            let mut s = String::new();
            let bytes_read = deflater.read_to_string(&mut s).map_err(|e| EmulError::State(StateError::Io(e)))?;
            info!("Read {} bytes from database file", bytes_read);
            serde_json::from_str(&s).map_err(|e| EmulError::State(StateError::Decoder(e)))
        })
    }

    /// open backend
    fn open(&self) -> Result<File, EmulError> {
        OpenOptions::new().create(true).read(true).write(true).open(self.path.as_path()).map_err(|e| EmulError::State(StateError::Io(e)))
    }

    /// mutate the file, always setting seek back to beginning
    fn mutate<F>(&self, mut fun: F) -> Result<(), EmulError>
    where
        F: FnMut(&mut File) -> Result<(), EmulError>
    {
        let mut file = self.open()?;
        fun(&mut file)?;
        file.seek(SeekFrom::Start(0)).map_err(|e| EmulError::State(StateError::Io(e)))?;
        Ok(())
    }

    /// read file, setting seek back to the start
    fn read<F>(&self, fun: F) -> Result<D, EmulError>
    where F: Fn(&File) -> Result<D, EmulError>
    {
        let mut file = self.open()?;
        let ret = fun(&file)?;
        file.seek(SeekFrom::Start(0)).map_err(|e| EmulError::State(StateError::Io(e)))?;
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn save() {
        pretty_env_logger::try_init();
        let db = SimpleDB::<HashMap<String, usize>>::new(PathBuf::from("/tmp/SOME")).unwrap();
        let mut data = HashMap::new();
        data.insert("Hello".to_string(), 45);
        data.insert("Byte".to_string(), 34);
        db.save(data.clone()).unwrap();
    }

    #[test]
    fn get() {
        pretty_env_logger::try_init();
        let db = SimpleDB::<HashMap<String, usize>>::new(PathBuf::from("/tmp/SOME")).unwrap();
        let mut data = HashMap::new();
        data.insert("Hello".to_string(), 45);
        data.insert("Byte".to_string(), 34);
        db.save(data.clone()).unwrap();
        info!("DATA: {:?}", db.get().unwrap());
    }
}
