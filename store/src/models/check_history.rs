use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable)]
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

use crate::store::Store;

impl Store {
    pub fn record_check(
        &mut self,
        website_id: String,
        is_up: bool,
        response_time_ms: Option<i32>,
        status_code: Option<i32>,
        error_message: Option<String>
    ) -> Result<CheckHistory, diesel::result::Error> {
        let id = Uuid::new_v4();
        let check = CheckHistory {
            id: id.to_string(),
            website_id,
            checked_at: chrono::Utc::now().naive_utc(),
            is_up,
            response_time_ms,
            status_code,
            error_message,

           };
           diesel::insert_into(crate::schema::check_history::table)
           .values(&check)
           .returning(CheckHistory::as_returning())
           .get_result(&mut self.conn)?;

        Ok(check)
    }

    pub fn update_website_status(
        &mut self,
        website_id: String,
        is_up_value: bool,
        response_time_ms_value: Option<i32>,
    ) -> Result<(), diesel::result::Error> {
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
        .execute(&mut self.conn)?;

        Ok(())
    }

    pub fn get_website_history(
        &mut self,
        website_id_value: String,
        limit: i64,
        offset: i64
    ) -> Result<Vec<CheckHistory>, diesel::result::Error> {
        use crate::schema::check_history::dsl::*;

        let results = check_history
        .filter(website_id.eq(website_id_value))
        .order(checked_at.desc())
        .limit(limit)
        .offset(offset)
        .select(CheckHistory::as_select())
        .load(&mut self.conn)?;
    Ok(results)
    }
}