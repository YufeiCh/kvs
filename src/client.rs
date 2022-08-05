use crate::common::{GetResponse, RemoveResponse, Request, SetResponse};
use std::io::{BufReader, Write};
use std::net::TcpStream;
use std::{io::BufWriter, net::ToSocketAddrs};

use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};

use crate::{KvsError, Result};

/// Key value store client.
pub struct KvsClient {
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    /// connect to addr to access from `KvsServer`.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let tcp_reader = TcpStream::connect(addr)?;
        let tcp_writer = tcp_reader.try_clone()?;
        Ok(KvsClient {
            writer: BufWriter::new(tcp_writer),
            reader: Deserializer::from_reader(BufReader::new(tcp_reader)),
        })
    }

    /// get the value of a given key from the server
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        serde_json::to_writer(&mut self.writer, &Request::Get { key })?;
        self.writer.flush()?;
        let resp = GetResponse::deserialize(&mut self.reader)?;
        match resp {
            GetResponse::Ok(value) => Ok(value),
            GetResponse::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }

    /// set the value of a string key in the server
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Set { key, value })?;
        self.writer.flush()?;
        let resp = SetResponse::deserialize(&mut self.reader)?;
        match resp {
            SetResponse::Ok(_) => Ok(()),
            SetResponse::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }

    /// remove a string key in the server
    pub fn remove(&mut self, key: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Remove { key })?;
        self.writer.flush()?;
        let resp = RemoveResponse::deserialize(&mut self.reader)?;
        match resp {
            RemoveResponse::Ok(_) => Ok(()),
            RemoveResponse::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }
}
