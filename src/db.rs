use std::env::VarError::{NotPresent, NotUnicode};

use libsql::{Builder, Database};
use tokio_util::bytes::Bytes;
use tracing::{info, warn};

const DATABASE_PATH: &str = "./db.sqlite";

pub async fn prepare() -> Database {
    let encryption_key = std::env::var("ENCRYPTION_KEY");

    match encryption_key {
        Ok(key) => {
            info!("Using provided database encryption key.");
            Builder::new_local(DATABASE_PATH)
                .encryption_config(libsql::EncryptionConfig {
                    cipher: libsql::Cipher::Aes256Cbc,
                    encryption_key: Bytes::from(key),
                })
                .build()
                .await
                .expect("Unable to initialize database!")
        }

        Err(NotPresent) => {
            warn!("DATABASE WILL NOT BE ENCRYPTED - No database encryption key supplied!");
            Builder::new_local(DATABASE_PATH)
                .build()
                .await
                .expect("Unable to initialize database!")
        }

        Err(NotUnicode(_)) => {
            panic!("Provided database encryption key is not valid UTF-8!")
        }
    }
}
