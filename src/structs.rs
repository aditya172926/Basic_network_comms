use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, process::exit, sync::RwLock};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Handshake {
        node_name: String,
        tcp_address: SocketAddr,
    },
    Greeting,
    Heartbeat,
    HeartbeatResponse,
    Setvalue {
        key: String,
        value: String,
    },
    Getvalue {
        key: String,
    },
    ValueResponse {
        value: Option<String>,
    },
    Sync {
        key: String,
        Value: String,
    },
}

struct KeyValueStore {
    store: RwLock<HashMap<String, String>>,
}

impl KeyValueStore {
    fn new() -> Self {
        KeyValueStore {
            store: RwLock::new(HashMap::new()),
        }
    }

    async fn set(&self, key: String, value: String) {
        let store = self.store.write();
        match store {
            Ok(mut object) => object.insert(key, value),
            Err(error) => {
                println!("Error in setting the value {:?}", error);
                exit(1);
            }
        };
        // store.insert(key, value);
    }

    async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read();
        let value = match store {
            Ok(object) => object.get(key).cloned()
            ,
            Err(error) => {
                println!("Error in setting the value {:?}", error);
                exit(1);
            }
        };
        value
    }
}
