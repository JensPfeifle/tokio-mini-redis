use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

// Commands to handle
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        connection_manager(rx).await;
    });

    let tx1 = tx.clone();
    let t1 = tokio::spawn(async move {
        let (rtx, rrx) = oneshot::channel();
        let cmd = Command::Get {
            key: "hello".to_string(),
            resp: rtx,
        };
        tx1.send(cmd).await.unwrap();

        let res = rrx.await;
        println!("GOT = {:?}", res);
    });

    let tx2 = tx.clone();
    let t2 = tokio::spawn(async move {
        let (rtx, rrx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: rtx,
        };
        tx2.send(cmd).await.unwrap();

        let res = rrx.await;
        println!("GOT = {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}

async fn connection_manager(mut rx: Receiver<Command>) {
    let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Get { key, resp } => {
                let val = client.get(&key).await;
                let _ = resp.send(val);
            }
            Command::Set { key, val, resp } => {
                let res = client.set(&key, val).await;
                let _ = resp.send(res);
            }
        }
    }
}
