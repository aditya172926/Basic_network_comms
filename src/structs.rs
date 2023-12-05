use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, time::Instant};
use std::{collections::HashMap, net::SocketAddr};

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
        value: String,
    },
}

pub struct KeyValueStore {
    store: RwLock<HashMap<String, String>>,
}

impl KeyValueStore {
    pub fn new() -> Self {
        KeyValueStore {
            store: RwLock::new(HashMap::new()),
        }
    }

    pub async fn set(&self, key: String, value: String) {
        let mut store = self.store.write().await;
        store.insert(key, value);
        // store.insert(key, value);
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        store.get(key).cloned()
    }
}

pub struct NodeInfo {
    pub last_seen: Instant,
    pub tcp_address: SocketAddr
}