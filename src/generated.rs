extern crate rmp_serde as rmps;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

extern crate log;
extern crate wapc_guest as guest;
use guest::prelude::*;

use lazy_static::lazy_static;
use std::sync::RwLock;

pub struct Host {
    binding: String,
}

impl Default for Host {
    fn default() -> Self {
        Host {
            binding: "default".to_string(),
        }
    }
}

/// Creates a named host binding
pub fn host(binding: &str) -> Host {
    Host {
        binding: binding.to_string(),
    }
}

/// Creates the default host binding
pub fn default() -> Host {
    Host::default()
}

impl Host {
    pub fn listdir(&self, dir: String) -> HandlerResult<Vec<String>> {
        let input_args = ListdirArgs { dir };
        host_call(&self.binding, "test", "listdir", &serialize(input_args)?)
            .map(|vec| {
                let resp = deserialize::<Vec<String>>(vec.as_ref()).unwrap();
                resp
            })
            .map_err(|e| e.into())
    }
}

pub struct Handlers {}

impl Handlers {
    pub fn register_listdir(f: fn(String) -> HandlerResult<Vec<String>>) {
        *LISTDIR.write().unwrap() = Some(f);
        register_function(&"listdir", listdir_wrapper);
    }
}

lazy_static! {
    static ref LISTDIR: RwLock<Option<fn(String) -> HandlerResult<Vec<String>>>> =
        RwLock::new(None);
}

fn listdir_wrapper(input_payload: &[u8]) -> CallResult {
    let input = deserialize::<ListdirArgs>(input_payload)?;
    let lock = LISTDIR.read().unwrap().unwrap();
    let result = lock(input.dir)?;
    Ok(serialize(result)?)
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct ListdirArgs {
    #[serde(rename = "dir")]
    pub dir: String,
}

/// The standard function for serializing codec structs into a format that can be
/// used for message exchange between actor and host. Use of any other function to
/// serialize could result in breaking incompatibilities.
pub fn serialize<T>(
    item: T,
) -> ::std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
where
    T: Serialize,
{
    let mut buf = Vec::new();
    item.serialize(&mut Serializer::new(&mut buf).with_struct_map())?;
    Ok(buf)
}

/// The standard function for de-serializing codec structs from a format suitable
/// for message exchange between actor and host. Use of any other function to
/// deserialize could result in breaking incompatibilities.
pub fn deserialize<'de, T: Deserialize<'de>>(
    buf: &[u8],
) -> ::std::result::Result<T, Box<dyn std::error::Error + Send + Sync>> {
    let mut de = Deserializer::new(Cursor::new(buf));
    match Deserialize::deserialize(&mut de) {
        Ok(t) => Ok(t),
        Err(e) => Err(format!("Failed to de-serialize: {}", e).into()),
    }
}
