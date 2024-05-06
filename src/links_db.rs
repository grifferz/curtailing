use sqlx::SqlitePool;

pub async fn populate(db_pool: &SqlitePool) -> Result<(), &str> {
    println!("->> Creating links table…");
    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS links (\
            uuid VARCHAR(36) PRIMARY KEY NOT NULL, \
            short VARCHAR(32) NOT NULL, \
            target VARCHAR(32768) NOT NULL\
        );",
    )
    .execute(db_pool)
    .await
    {
        Ok(result) => result,
        Err(e) => {
            panic!("{}", e);
        }
    };

    println!("->> Creating UNIQUE index on links.short…");
    match sqlx::query("CREATE UNIQUE INDEX short_idx on links(short);")
        .execute(db_pool)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            panic!("{}", e);
        }
    };

    println!("->> INSERT first link…");
    match sqlx::query(
        "INSERT INTO links
            (uuid, short, target)
        VALUES (?, ?, ?)",
    )
    .bind("018f244b-942b-7007-927b-ace4fadf4a88")
    .bind("6fy")
    .bind(
        "https://mailman.bitfolk.com/mailman/hyperkitty/list/\
            users@mailman.bitfolk.com/message/\
            BV6BHVJN7YL4OYN7C5Y5LRPWJKALPWY6/",
    )
    .execute(db_pool)
    .await
    {
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
