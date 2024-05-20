mod db;

#[tokio::main]
async fn main() {
    println!("Starting...");

    // delete the db file if it exists
    let db_path = std::env::current_dir().unwrap().join("gdl_conf.db");

    if db_path.exists() {
        std::fs::remove_file(&db_path).unwrap();
    }

    let time_before = std::time::Instant::now();
    let db_uri = format!(
        "file:{}?connection_limit=1",
        std::env::current_dir()
            .unwrap()
            .join("gdl_conf.db")
            .to_str()
            .unwrap()
    );

    let db_client = db::new_client_with_url(&db_uri).await.unwrap();

    db_client._migrate_deploy().await.unwrap();

    let time_after = std::time::Instant::now();

    println!("Time taken: {:?}", time_after - time_before);

    std::thread::sleep(std::time::Duration::from_secs(15));
}
