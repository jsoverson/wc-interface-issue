#[macro_use]
extern crate log;

use blob::Container;
use guest::prelude::*;
use wapc_guest as guest;
use wasmcloud_actor_blobstore as blob;
use wasmcloud_actor_core as actor;

mod generated;

#[actor::init]
fn init() {
    generated::Handlers::register_listdir(test);
}

fn test(dir: String) -> HandlerResult<Vec<String>> {
    let container = Container { id: dir };
    match blob::default().list_objects(container) {
        Ok(blob_list) => Ok(blob_list.blobs.into_iter().map(|blob| blob.id).collect()),
        Err(e) => {
            error!("could not retrieve objects");
            Err(e)
        }
    }
}
