use std::vec;

use wring::fs::File;

use std::{
    io::{Read, Seek},
    sync::Arc,
};

use tokio::{
    io::{AsyncReadExt, AsyncSeekExt},
    task::JoinSet,
};

const FILE: &str = "../random.txt";
const ITERATIONS: usize = 1000;
const SIZE: usize = 8 * 1024;

#[tokio::main]
async fn main() {
    wring::start(4096);

    let file = File::open(FILE).await.unwrap();
    let file = Arc::new(file);

    let buf = vec![0; SIZE];

    let start = std::time::Instant::now();

    let mut set = JoinSet::new();

    for _ in 0..ITERATIONS {
        let file = file.clone();
        let buf = buf.clone();
        set.spawn(async move {
            let (res, _buf) = file.read(buf, 0).await;
            res.unwrap();
        });
    }

    let mut i = 0;
    while let Some(_) = set.join_next().await {
        i += 1;
    }

    let elapsed = start.elapsed();
    println!("tokiour batch elapsed: {:?}", elapsed);
}

#[cfg(test)]
mod test {

    use std::vec;

    use wring::fs::File;

    use std::{
        io::{Read, Seek},
        sync::Arc,
    };

    use tokio::{
        io::{AsyncReadExt, AsyncSeekExt},
        task::JoinSet,
    };

    const FILE: &str = "../random.txt";
    const ITERATIONS: usize = 40000;
    const SIZE: usize = 8 * 1024;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn tokiour_read() {
        wring::start(32);

        let file = File::open(FILE).await.unwrap();

        let mut buf = vec![0; SIZE];

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let (_, out) = file.read(buf, 0).await;
            buf = out;
        }
        let elapsed = start.elapsed();
        println!("tokiour elapsed: {:?}", elapsed);
    }

    // #[tokio::test(flavor = "multi_thread", worker_threads = 6)]
    // async fn tokiour_read_batch() {
    //     wring::start(4096);

    //     let file = File::open(FILE).await.unwrap();
    //     let file = Arc::new(file);

    //     let buf = vec![0; SIZE];

    //     let start = std::time::Instant::now();

    //     let mut set = JoinSet::new();

    //     for _ in 0..ITERATIONS {
    //         let file = file.clone();
    //         let buf = buf.clone();
    //         set.spawn(async move {
    //             let (res, _) = file.read_at(buf, 0).await;
    //             res.unwrap();
    //         });
    //     }

    //     set.join_all().await;

    //     let elapsed = start.elapsed();
    //     println!("tokiour batch elapsed: {:?}", elapsed);
    // }

    #[test]
    fn std_read() {
        let mut file = std::fs::File::open(FILE).unwrap();
        let mut buf = vec![0; SIZE];

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            file.seek(std::io::SeekFrom::Start(0)).unwrap();
            file.read(&mut buf).unwrap();
        }
        let elapsed = start.elapsed();
        println!("std elapsed: {:?}", elapsed);
    }

    #[tokio::test]
    async fn tokio_read() {
        let mut file = tokio::fs::File::open(FILE).await.unwrap();
        let mut buf = vec![0; SIZE];

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            file.seek(std::io::SeekFrom::Start(0)).await.unwrap();
            file.read(buf.as_mut_slice()).await.unwrap();
        }
        let elapsed = start.elapsed();
        println!("tokio elapsed: {:?}", elapsed);
    }

    #[test]
    fn tokio_uring_read() {
        tokio_uring::start(async {
            let file = tokio_uring::fs::File::open(FILE).await.unwrap();

            let mut buf = vec![0; SIZE];

            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let (_, out) = file.read_at(buf, 0).await;
                buf = out;
            }
            let elapsed = start.elapsed();
            println!("tokio-uring elapsed: {:?}", elapsed);
        });
    }
}
