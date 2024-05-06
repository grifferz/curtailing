use crate::links_db::count_links;
use crate::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

// Maximum length of a target URL in bytes. RFC allows longer than this but most CDNs don't, so
// let's use that as the limit.
const MAX_URL_LENGTH: usize = 3072;

// How many times to try inserting a new link when a UNIQUE index constraint on `short` is hit.
const MAX_CREATE_TRIES: usize = 1000;

// Model a row from the database, also serialize to JSON for return from API for the time being -
// in a production version we don't want to be exposing the UUIDs that are the database primary
// key..
#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct Link {
    pub uuid: String,
    pub short: String,
    pub target: String,
}

// This will be parsed from the JSON that the user supplies when they visit the create_link
// handler.
#[derive(Deserialize)]
pub struct LinkForCreate {
    pub target: String,
}

// State to be available in all requests. Just the database pool connection at the moment. Later
// there might be something for a parsed config file or similar.
#[derive(Clone)]
pub struct ModelController {
    db_pool: SqlitePool,
}

impl ModelController {
    pub async fn new(db_pool: SqlitePool) -> Result<Self> {
        Ok(Self {
            db_pool: db_pool.clone(),
        })
    }
}

impl ModelController {
    // Add a new short link.
    pub async fn create_link(&self, link_fc: LinkForCreate) -> Result<Link> {
        let db_pool = self.db_pool.clone();

        if link_fc.target.is_empty() {
            return Err(Error::LinkCreateFailedEmptyTarget);
        }

        if link_fc.target.len() > MAX_URL_LENGTH {
            return Err(Error::LinkCreateFailedTargetTooLong);
        }

        // Only allow http and https links.
        let re = Regex::new("^(?i)https?://").unwrap();

        if !re.is_match(&link_fc.target) {
            return Err(Error::LinkCreateFailedBadProtocol);
        }

        let existing_link_count = match count_links(&db_pool).await {
            Ok(count) => count,
            Err(e) => {
                // Just return some generic database error.
                eprintln!(
                    "Link creation failed due to unhandled database error: {:?}",
                    e
                );
                return Err(Error::LinkCreateFailedDBError);
            }
        };

        if existing_link_count >= 65 {
            eprintln!("TODO: Can't yet handle more than 65 links in the database");
            return Err(Error::LinkCreateFailedDBError);
        }

        let mut try_count = 0;

        // Collision chance for a 16-bit perfect hash given n tries according to:
        //      https://en.wikipedia.org/wiki/Birthday_attack#Mathematics
        // approximates to:
        //      p = 1 - e^-(n * ((n-1) / (2*2^16)))
        //
        // So for two tries:
        //      p = 1 - e^-(2 * (1 / 131072)
        //        = 1 - e^-(2 * 0.000007629)
        //        = 1 - e^-0.000015259
        //        = 1 - 0.999984741
        //        = 0.000015259 or 0.0015259 %
        //
        // But we are going to allow up to 65 short links for 16 bits (before we expand to 24 bits
        // and beyond) so that means the collision chance for that last SQL insert with 64 links
        // already in the DB would be:
        //      p = 1 - e^-(65 * (64 / 131072)
        //        = 1 - e^-(65 * 0.000488281)
        //        = 1 - e^-0.031738281
        //        = 1 - 0.968760092
        //        = 0.031239908 or 3.1239908 %
        //
        // So let's just give it 1,000 tries to insert. If we can't do it in 1,000 tries then
        // something weird is going on.
        //
        // However, if our intention is to expand to 24 bits for up to 16,776 links then the worst
        // case on the insert of that last 16776th link would be:
        //
        //      p = 1 - e^-(n * ((n-1) / (2*2^24)))
        //        = 1 - e^-(16776 * ((16775) / (2*2^24)))
        //        = 1 - e^-(16776 * (16775 / 33554432))
        //        = 1 - e^-(16776 * 0.000499934)
        //        - 1 - e^-8.386892784
        //        = 1 - 0.000227834
        //        = 0.999772166 or 99.9772166% %
        // which is far too high!
        //
        // On a 24 bit space, probability of collision is about 50% at 4,822 tries, so if we want
        // to keep collision chance below 50% here are the limits on number of links for each bit
        // range:
        //
        // bits | number of tries before collision chance becomes >= 50%
        // -----|-------------------------------------------------------
        // 16   |           301
        // 24   |         4,822
        // 32   |        77,162
        // 40   |     1,234,603
        // 48   |    19,753,662
        // 56   |   316,058,596
        // 62   | 2,528,468,770 (We don't have a full 64 bits due to 2 bits of the UUID being used
        //                       for versioning)
        while try_count < MAX_CREATE_TRIES {
            let uuid = Uuid::now_v7();
            // TODO: Need to use more than 16 bits when there's more links in the DB.
            let lower_16b = &uuid.as_bytes()[14..];
            let base58 = bs58::encode(lower_16b).into_string();

            let link = Link {
                short: base58,
                target: link_fc.target.clone(),
                uuid: uuid.to_string(),
            };

            let result = sqlx::query(
                "INSERT INTO links (uuid, short, target)
                VALUES ($1, $2, $3)",
            )
            .bind(&link.uuid)
            .bind(&link.short)
            .bind(&link.target)
            .execute(&db_pool)
            .await;

            match result {
                Ok(_) => {
                    if try_count > 0 {
                        eprintln!(
                            "Link for {} created after {} collision{}",
                            link.short,
                            try_count,
                            if try_count == 1 { "" } else { "s" }
                        );
                    }
                    return Ok(link);
                }
                Err(e) => match e {
                    // Check for a Database error with code 2067 which is the SQLite error code for
                    // unique constraint failed. Keep trying to create links if there was a
                    // collision.
                    // I don't know why this needs `std::borrow::Cow::Borrowed()`.
                    sqlx::Error::Database(se)
                        if se.code() == Some(std::borrow::Cow::Borrowed("2067")) =>
                    {
                        try_count += 1;

                        if try_count >= MAX_CREATE_TRIES {
                            // This is going to return an error and give up, so log a bit more
                            // detail.
                            eprintln!(
                                "Too many collisions ({}) on short link ({})",
                                try_count, link.short
                            );
                        }
                    }
                    // Just return some generic database error to the user.
                    _ => {
                        eprintln!(
                            "Link creation failed due to unhandled database error: {:?}",
                            e
                        );
                        return Err(Error::LinkCreateFailedDBError);
                    }
                },
            }
        }

        return Err(Error::LinkCreateFailedTooManyCollisions);
    }
}

impl ModelController {
    pub async fn get_link(&self, short: &str) -> Result<Link> {
        let db_pool = self.db_pool.clone();

        let link = match sqlx::query_as::<_, Link>(
            "SELECT uuid, short, target
            FROM links
            WHERE short = $1",
        )
        .bind(&short)
        .fetch_optional(&db_pool)
        .await
        {
            Ok(Some(l)) => l,
            Ok(None) => {
                // Successful query but no rows returned, so that's a 404.
                return Err(Error::LinkGetNoSuchLink);
            }
            Err(e) => {
                eprintln!(
                    "Failed to get short link due to unhandled database error: {:?}",
                    e
                );
                return Err(Error::LinkGetDBError);
            }
        };

        Ok(link)
    }
}
impl ModelController {
    pub async fn list_links(&self) -> Result<Vec<Link>> {
        let db_pool = self.db_pool.clone();

        let links = sqlx::query_as::<_, Link>(
            "SELECT uuid, short, target
            FROM links",
        )
        .fetch_all(&db_pool)
        .await
        .unwrap();

        Ok(links)
    }
}
