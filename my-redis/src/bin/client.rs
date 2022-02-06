use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

// Commands to handle
#[derive(Debug)]
enum Command {
    Get { key: String },
    Set { key: String, val: Bytes },
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        connection_manager(rx).await;
    });

    let tx1 = tx.clone();
    let t1 = tokio::spawn(async move {
        let cmd = Command::Get {
            key: "hello".to_string(),
        };
        tx1.send(cmd).await.unwrap()
    });

    let tx2 = tx.clone();
    let t2 = tokio::spawn(async move {
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
        };
        tx2.send(cmd).await.unwrap()
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}

async fn connection_manager(mut rx: Receiver<Command>) {
    let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Get { key } => {
                client.get(&key).await;
            }
            Command::Set { key, val } => {
                client.set(&key, val).await;
            }
        }
    }
}
