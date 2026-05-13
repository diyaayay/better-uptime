use crate::store::{Store, StoreError};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::check_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CheckHistory {
    pub id: String,
    pub website_id: String,
    pub checked_at: chrono::NaiveDateTime,
    pub is_up: bool,
    pub response_time_ms: Option<i32>,
    pub status_code: Option<i32>,
    pub error_message: Option<String>,
}

impl Store {
    pub async fn record_check(
        &self,
        website_id: String,
        is_up: bool,
        response_time_ms: Option<i32>,
        status_code: Option<i32>,
        error_message: Option<String>,
    ) -> Result<CheckHistory, StoreError> {
        let check = CheckHistory {
            id: Uuid::new_v4().to_string(),
            website_id,
            checked_at: chrono::Utc::now().naive_utc(),
            is_up,
            response_time_ms,
            status_code,
            error_message,
        };

        let conn = self.pool.get().await?;
        let inserted = conn
            .interact(move |conn| {
                diesel::insert_into(crate::schema::check_history::table)
                    .values(&check)
                    .returning(CheckHistory::as_returning())
                    .get_result(conn)
            })
            .await??;

        Ok(inserted)
    }

    pub async fn update_website_status(
        &self,
        website_id: String,
        is_up_value: bool,
        response_time_ms_value: Option<i32>,
    ) -> Result<(), StoreError> {
        let conn = self.pool.get().await?;
        conn.interact(move |conn| {
            use crate::schema::website::dsl::*;
            let now = chrono::Utc::now().naive_utc();
            let last_down_time_value = if !is_up_value { Some(now) } else { None };

            diesel::update(website.filter(id.eq(website_id)))
                .set((
                    is_up.eq(Some(is_up_value)),
                    last_checked.eq(Some(now)),
                    last_down_time.eq(last_down_time_value),
                    response_time_ms.eq(response_time_ms_value),
                ))
                .execute(conn)
        })
        .await??;

        Ok(())
    }

    pub async fn get_website_history(
        &self,
        website_id_value: String,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CheckHistory>, StoreError> {
        let conn = self.pool.get().await?;
        let results = conn
            .interact(move |conn| {
                use crate::schema::check_history::dsl::*;
                check_history
                    .filter(website_id.eq(website_id_value))
                    .order(checked_at.desc())
                    .limit(limit)
                    .offset(offset)
                    .select(CheckHistory::as_select())
                    .load(conn)
            })
            .await??;
        Ok(results)
    }
}
