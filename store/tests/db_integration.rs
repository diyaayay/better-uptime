//! Postgres integration tests for [`store::Store`].
//! Requires `DATABASE_URL` (CI sets it; locally load repo `.env` from the workspace root).

use std::time::Duration;

use diesel::Connection;
use diesel::pg::PgConnection;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use store::{Store, StoreError};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn load_env() -> String {
    let workspace_env = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
    let _ = dotenvy::from_path(&workspace_env);
    let _ = dotenvy::from_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env"));
    std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. GitHub Actions or `docker compose` + `.env`)")
}

fn run_migrations(database_url: &str) {
    let mut conn = PgConnection::establish(database_url).expect("connect for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("run_pending_migrations");
}

fn test_password_hash(plain: &str) -> String {
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};
    let salt = SaltString::generate(&mut rand::thread_rng());
    Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .expect("hash test password")
        .to_string()
}

fn unique_name(prefix: &str) -> String {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}_{n}")
}

async fn connect_store() -> Store {
    let url = load_env();
    run_migrations(&url);
    Store::connect(&url).expect("Store::connect")
}

#[tokio::test]
async fn ping_db() {
    let store = connect_store().await;
    store.ping_db().await.expect("ping_db");
}

#[tokio::test]
async fn sign_up_sign_in_wrong_password_and_duplicate_username() {
    let store = connect_store().await;
    let username = unique_name("dbuser");
    let plain = "longenough";
    let hash = test_password_hash(plain);

    let id = store
        .sign_up(username.clone(), hash.clone())
        .await
        .expect("sign_up");
    assert!(!id.is_empty());

    let signed_in = store
        .sign_in(username.clone(), plain.to_string())
        .await
        .expect("sign_in");
    assert_eq!(signed_in, id);

    let wrong = store
        .sign_in(username.clone(), "other-pass".to_string())
        .await;
    assert!(matches!(
        wrong,
        Err(StoreError::Diesel(DieselError::NotFound))
    ));

    let dup = store.sign_up(username, hash).await;
    assert!(matches!(
        dup,
        Err(StoreError::Diesel(DieselError::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            _
        )))
    ));
}

#[tokio::test]
async fn website_crud_list_delete_and_not_found() {
    let store = connect_store().await;
    let username = unique_name("webuser");
    let user_id = store
        .sign_up(username, test_password_hash("password12"))
        .await
        .expect("sign_up");

    let w = store
        .create_website(user_id.clone(), "https://example.com/a".into())
        .await
        .expect("create_website");
    assert_eq!(w.user_id, user_id);
    assert_eq!(w.url, "https://example.com/a");

    let fetched = store.get_website(w.id.clone()).await.expect("get_website");
    assert_eq!(fetched.id, w.id);

    let list = store
        .list_websites(user_id.clone())
        .await
        .expect("list_websites");
    assert!(list.iter().any(|x| x.id == w.id));

    let updated = store
        .update_website(
            w.id.clone(),
            user_id.clone(),
            "https://example.com/b".into(),
        )
        .await
        .expect("update_website");
    assert_eq!(updated.url, "https://example.com/b");

    let other_user = store
        .sign_up(unique_name("other"), test_password_hash("password12"))
        .await
        .expect("other user");

    let wrong_owner = store.delete_website(w.id.clone(), other_user.clone()).await;
    assert!(matches!(
        wrong_owner,
        Err(StoreError::Diesel(DieselError::NotFound))
    ));

    let wrong_update = store
        .update_website(w.id.clone(), other_user, "https://evil.com".into())
        .await;
    assert!(matches!(
        wrong_update,
        Err(StoreError::Diesel(DieselError::NotFound))
    ));

    let deleted = store
        .delete_website(w.id.clone(), user_id.clone())
        .await
        .expect("delete_website");
    assert_eq!(deleted, 1);

    let gone = store.get_website(w.id).await;
    assert!(matches!(
        gone,
        Err(StoreError::Diesel(DieselError::NotFound))
    ));
}

#[tokio::test]
async fn check_history_and_website_status() {
    let store = connect_store().await;
    let user_id = store
        .sign_up(unique_name("histuser"), test_password_hash("password12"))
        .await
        .expect("sign_up");

    let w = store
        .create_website(user_id, "https://example.com/monitor".into())
        .await
        .expect("create_website");

    let row = store
        .record_check(w.id.clone(), true, Some(42), Some(200), None)
        .await
        .expect("record_check");
    assert_eq!(row.website_id, w.id);
    assert!(row.is_up);

    tokio::time::sleep(Duration::from_millis(5)).await;

    let hist = store
        .get_website_history(w.id.clone(), 10, 0)
        .await
        .expect("get_website_history");
    assert!(
        hist.iter().any(|h| h.id == row.id),
        "history should include the inserted check"
    );

    store
        .update_website_status(w.id.clone(), false, Some(100))
        .await
        .expect("update_website_status");

    let after = store
        .get_website(w.id)
        .await
        .expect("get_website after status");
    assert_eq!(after.is_up, Some(false));
    assert_eq!(after.response_time_ms, Some(100));
    assert!(after.last_checked.is_some());
    assert!(after.last_down_time.is_some());
}
