use wring::fs::File;

#[tokio::main]
async fn main() {
    let mut _guard = wring::background(64).unwrap();
    let buf = Vec::with_capacity(1024);

    let file = File::open("Cargo.toml").await.unwrap();
    let (res, buf) = file.read(buf, 0).await;
    res.unwrap();

    let text = String::from_utf8(buf).unwrap();
    println!("{}", text);
}
