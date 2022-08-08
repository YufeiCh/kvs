use std::borrow::BorrowMut;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::{KvsEngine, KvsError, Result};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// The KvStore stores string key/value pairs
///
/// Example:
///
/// ```rust
/// # use kvs::{KvStore, Result};
/// # use std::env::current_dir;
/// fn try_main() -> Result<()> {
/// # use kvs::KvsEngine;
/// # let mut store = KvStore::open(current_dir()?)?;
/// # store.set("key1".to_owned(), "value1".to_owned());
/// # let val = store.get("key1".to_owned())?;
/// # assert_eq!(val, Some("value1".to_owned()));
/// # Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct KvStore {
    index: Arc<Mutex<BTreeMap<String, CommandPos>>>,
    readers: Arc<Mutex<HashMap<u64, BufReaderWithPos<File>>>>,
    writer: Arc<Mutex<BufWriterWithPos<File>>>,
    path: Arc<PathBuf>,
    current_gen: Arc<Mutex<u64>>,
    uncompacted: Arc<Mutex<u64>>,
}

impl KvStore {
    /// new a KvStore with the log in the specific filePath
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = Arc::new(path.into());
        fs::create_dir_all(&*path)?;
        let mut index = BTreeMap::new();
        let mut readers = HashMap::new();
        let mut uncompacted = 0;

        let gen_list = sorted_gen_list(&path)?;
        for &gen in &gen_list {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path, gen))?)?;
            uncompacted += load(gen, &mut reader, &mut index)?;
            readers.insert(gen, reader);
        }
        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, current_gen, &mut readers)?;
        Ok(KvStore {
            index: Arc::new(Mutex::new(index)),
            readers: Arc::new(Mutex::new(readers)),
            writer: Arc::new(Mutex::new(writer)),
            path,
            current_gen: Arc::new(Mutex::new(current_gen)),
            uncompacted: Arc::new(Mutex::new(uncompacted)),
        })
    }

    /// compact log when reach the thereshold
    pub fn compact(&self) -> Result<()> {
        let mut current_gen = self.current_gen.lock().unwrap();
        let mut writer = self.writer.lock().unwrap();
        let compaction_gen = *current_gen + 1;
        *current_gen += 2;
        *writer = self.new_log_file(*current_gen)?;

        let mut compaction_writer = self.new_log_file(compaction_gen)?;

        let mut new_pos = 0;
        let mut index = self.index.lock().unwrap();
        let mut readers = self.readers.lock().unwrap();
        for cmd_pos in index.values_mut() {
            let reader = readers
                .get_mut(&cmd_pos.gen)
                .expect("cannot find log reader");
            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut entry_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut compaction_writer)?;
            *cmd_pos = (compaction_gen, new_pos..new_pos + len).into();
            new_pos += len;
        }

        compaction_writer.flush()?;

        let stale_gens: Vec<_> = readers
            .keys()
            .filter(|&&gen| gen < compaction_gen)
            .cloned()
            .collect();

        for stale_gen in stale_gens {
            readers.remove(&stale_gen);
            fs::remove_file(log_path(&self.path, stale_gen))?;
        }

        Ok(())
    }

    /// create a new log file with the given gen and add the file reader to the reader map
    ///
    /// Reture the writer
    fn new_log_file(&self, gen: u64) -> Result<BufWriterWithPos<File>> {
        let mut readers = self.readers.lock().unwrap();
        new_log_file(&self.path, gen, &mut readers)
    }
}

impl KvsEngine for KvStore {
    fn set(&self, key: String, value: String) -> Result<()> {
        let set_command = Command::set(key, value);
        let mut uncompacted = self.uncompacted.lock().unwrap();
        let current_gen = self.current_gen.lock().unwrap();
        {
            let mut writer = self.writer.lock().unwrap();
            let mut index = self.index.lock().unwrap();
            let pos = writer.pos;
            serde_json::to_writer(&mut *writer, &set_command)?;
            writer.flush()?;
            if let Command::Set { key, .. } = set_command {
                if let Some(old_cmd) = index
                    .insert(key, (*current_gen, pos..writer.pos).into())
                {
                    *uncompacted += old_cmd.len;
                }
            }
        }
        if *uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    fn remove(&self, key: String) -> Result<()> {
        let mut index = self.index.lock().unwrap();
        if index.contains_key(&key) {
            let cmd = Command::remove(key);
            {
                let mut writer = self.writer.lock().unwrap();
                serde_json::to_writer(&mut *writer, &cmd)?;
                writer.flush()?;
                if let Command::Remove { key } = cmd {
                    let old_cmd = index.remove(&key).expect("Key not found");
                    let mut uncompacted = self.uncompacted.lock().unwrap();
                    *uncompacted += old_cmd.len;
                }
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let index = self.index.lock().unwrap();
        if let Some(cmd) = index.get(&key) {
            let mut readers = self.readers.lock().unwrap();
            let reader = readers.get_mut(&cmd.gen).expect("Cannot find reader");
            reader.seek(SeekFrom::Start(cmd.pos))?;
            let cmd_reader = reader.take(cmd.len);
            if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }
}

fn sorted_gen_list(path: &Path) -> Result<Vec<u64>> {
    let mut gen_list: Vec<u64> = fs::read_dir(path)?
        .flat_map(|entry| -> Result<_> { Ok(entry?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    gen_list.sort_unstable();
    Ok(gen_list)
}

fn log_path(dir: &Path, gen: u64) -> PathBuf {
    dir.join(format!("{}.log", gen))
}

fn new_log_file(
    dir: &Path,
    gen: u64,
    readers: &mut HashMap<u64, BufReaderWithPos<File>>,
) -> Result<BufWriterWithPos<File>> {
    let path = log_path(dir, gen);
    let writer = BufWriterWithPos::new(
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?,
    )?;
    readers.insert(gen, BufReaderWithPos::new(File::open(&path)?)?);
    Ok(writer)
}

fn load(
    gen: u64,
    reader: &mut BufReaderWithPos<File>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut uncompacted = 0;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();

    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) = index.insert(key, (gen, pos..new_pos).into()) {
                    uncompacted += old_cmd.len;
                }
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.len;
                }
                uncompacted += new_pos - pos;
            }
        }
        pos = new_pos;
    }

    Ok(uncompacted)
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

#[derive(Clone)]
struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((gen, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            gen,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}
