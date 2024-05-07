use sqlx::SqlitePool;

pub async fn populate(db_pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations_dir = std::path::Path::new(&crate_dir).join("./migrations");
    let m = sqlx::migrate::Migrator::new(migrations_dir).await?;

    for migration in m.iter() {
        println!(
            "->> Applying DB migration version {} [{}]",
            migration.version, migration.description
        );
    }

    match m.run(db_pool).await {
        Ok(result) => result,
        Err(e) => {
            panic!("{}", e);
        }
    };

    Ok(())
}

// Count how many links are in our database already. This is needed because
// we start off using a small hash (16 bits) and expand it as necessary.
pub async fn count_links(db_pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(uuid)
         FROM links",
    )
    .fetch_one(db_pool)
    .await?;

    Ok(count)
}
