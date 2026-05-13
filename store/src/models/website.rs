use crate::store::{Store, StoreError};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::website)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Website {
    pub id: String,
    pub url: String,
    pub user_id: String,
    pub time_added: chrono::NaiveDateTime,
    pub is_up: Option<bool>,
    pub last_checked: Option<chrono::NaiveDateTime>,
    pub last_down_time: Option<chrono::NaiveDateTime>,
    pub response_time_ms: Option<i32>,
    pub webhook_url: Option<String>,
}

impl Store {
    pub async fn create_website(
        &self,
        user_id: String,
        url: String,
        webhook_url: Option<String>,
    ) -> Result<Website, StoreError> {
        let new_website = Website {
            id: Uuid::new_v4().to_string(),
            url,
            user_id,
            time_added: Utc::now().naive_utc(),
            is_up: Some(true),
            last_checked: None,
            last_down_time: None,
            response_time_ms: None,
            webhook_url,
        };

        let conn = self.pool.get().await?;
        let inserted = conn
            .interact(move |conn| {
                diesel::insert_into(crate::schema::website::table)
                    .values(&new_website)
                    .returning(Website::as_returning())
                    .get_result(conn)
            })
            .await??;

        Ok(inserted)
    }

    pub async fn get_website(&self, input_id: String) -> Result<Website, StoreError> {
        let conn = self.pool.get().await?;
        let website_result = conn
            .interact(move |conn| {
                use crate::schema::website::dsl::*;
                website
                    .filter(id.eq(input_id))
                    .select(Website::as_select())
                    .first(conn)
            })
            .await??;
        Ok(website_result)
    }

    pub async fn list_websites(&self, input_user_id: String) -> Result<Vec<Website>, StoreError> {
        let conn = self.pool.get().await?;
        let websites = conn
            .interact(move |conn| {
                use crate::schema::website::dsl::*;
                website
                    .filter(user_id.eq(input_user_id))
                    .order(time_added.desc())
                    .select(Website::as_select())
                    .load(conn)
            })
            .await??;
        Ok(websites)
    }

    pub async fn update_website(
        &self,
        website_id: String,
        input_user_id: String,
        new_url: String,
        webhook_url_patch: Option<Option<String>>,
    ) -> Result<Website, StoreError> {
        let conn = self.pool.get().await?;
        let updated = conn
            .interact(move |conn| {
                use crate::schema::website::dsl::*;
                match webhook_url_patch {
                    Some(wh) => diesel::update(website)
                        .filter(id.eq(website_id.clone()))
                        .filter(user_id.eq(input_user_id.clone()))
                        .set((url.eq(new_url.clone()), webhook_url.eq(wh)))
                        .returning(Website::as_returning())
                        .get_result(conn),
                    None => diesel::update(website)
                        .filter(id.eq(website_id))
                        .filter(user_id.eq(input_user_id))
                        .set(url.eq(new_url))
                        .returning(Website::as_returning())
                        .get_result(conn),
                }
            })
            .await??;
        Ok(updated)
    }

    pub async fn delete_website(
        &self,
        website_id: String,
        input_user_id: String,
    ) -> Result<usize, StoreError> {
        let conn = self.pool.get().await?;
        let deleted = conn
            .interact(move |conn| {
                use crate::schema::website::dsl::*;
                diesel::delete(website)
                    .filter(id.eq(website_id))
                    .filter(user_id.eq(input_user_id))
                    .execute(conn)
            })
            .await??;

        if deleted == 0 {
            return Err(StoreError::Diesel(DieselError::NotFound));
        }
        Ok(deleted)
    }

    pub async fn get_all_websites(&self) -> Result<Vec<Website>, StoreError> {
        let conn = self.pool.get().await?;
        let all_websites = conn
            .interact(|conn| {
                use crate::schema::website::dsl::*;
                website.select(Website::as_select()).load(conn)
            })
            .await??;
        Ok(all_websites)
    }
}
