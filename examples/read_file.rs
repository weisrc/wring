use wring::fs::File;

#[tokio::main]
async fn main() {
    let mut _bg = wring::Background::new(64).unwrap();

    let mut buf = Vec::with_capacity(1024);

    let file = File::open("Cargo.toml").await.unwrap();
    file.read(&mut buf, 0).await.unwrap();

    let text = String::from_utf8(buf).unwrap();
    println!("{}", text);
}
