use std::borrow::Borrow;

use sha2::{Digest, Sha384};
use sqlx::{
    error::{DatabaseError, ErrorKind},
    migrate::{self, MigrateDatabase},
    Executor, Sqlite, SqlitePool,
};

struct SqlxMigration {
    version: String,
    description: String,
    installed_on: String,
    success: bool,
    checksum: String,
    execution_time: i32,
}

#[tokio::main]
async fn main() {
    println!("Starting....");

    // delete the db file if it exists
    let db_path = env!("CARGO_MANIFEST_DIR").to_string() + "/gdl_conf.db";
    // std::env::current_dir().unwrap().join("gdl_conf.db");

    // if db_path.exists() {
    //     std::fs::remove_file(&db_path).unwrap();
    // }

    if !Sqlite::database_exists(&db_path).await.unwrap_or(false) {
        println!("Creating database {}", db_path);
        match Sqlite::create_database(&db_path).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    }

    let time_before = std::time::Instant::now();

    let db = SqlitePool::connect(&*format!("{}?connection_limit=1", &db_path)).await.unwrap();

    let new_migrations = sqlx::migrate!("./prisma/migrations");

    let migrations = sqlx::query("SELECT * FROM _prisma_migrations")
        .fetch_all(&db)
        .await
        .unwrap();

    let sqlx_migrations = sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        );
    "#,
    )
    .execute(&db)
    .await
    .unwrap();

    for m in migrations {
        let timestamp = &m.migration_name.split('_').collect::<Vec<&str>>()[0]
            .parse::<i64>()
            .unwrap();

        let migrator_entry = new_migrations
            .iter()
            .find(|m| &m.version == timestamp)
            .unwrap();

        let result = sqlx::query(
            r#"
            INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
            VALUES (?, ?, ?, ?, ?)
        "#,
        )
        .bind(migrator_entry.version)
        .bind(migrator_entry.description.clone())
        .bind(true)
        .bind(migrator_entry.checksum.to_vec())
        .bind(0)
        .execute(&db)
        .await;

        if let Err(error) = result {
            match error {
                sqlx::Error::Database(db_error) => {
                    if db_error.is_unique_violation() {
                        continue;
                    }
                }
                _ => panic!("error: {}", error),
            }
        }
    }

    sqlx::query(
        r#"
            DROP TABLE _prisma_migrations;
        "#,
    )
    .execute(&db)
    .await
    .unwrap();

    let migration_results = new_migrations.run(&db).await;

    match migration_results {
        Ok(_) => println!("Migration success"),
        Err(error) => {
            panic!("error: {}", error);
        }
    }

    let time_after = std::time::Instant::now();

    println!("Time taken: {:?}", time_after - time_before);
}
