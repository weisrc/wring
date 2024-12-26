use wring::{fs::File, io::FixedPool};

#[tokio::main]
async fn main() {
    let mut _bg = wring::Background::new(64).unwrap();

    let bufs: Vec<Vec<u8>> = (0..64).map(|_| Vec::with_capacity(1024)).collect();
    let buf_pool = FixedPool::new(bufs.into_iter());

    let mut buf = buf_pool.get().unwrap();
    let file = File::open("Cargo.toml").await.unwrap();
    file.read_fixed(&mut buf, 0).await.unwrap();

    let text = String::from_utf8(buf.clone()).unwrap();

    drop(_bg);
    drop(buf_pool);
    println!("{}", text);
}
