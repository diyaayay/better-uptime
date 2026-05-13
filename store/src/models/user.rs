use crate::store::{Store, StoreError};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    username: String,
    password: String,
}

impl Store {
    pub async fn sign_up(&self, username: String, password: String) -> Result<String, StoreError> {
        let id = Uuid::new_v4().to_string();
        let new_user = User {
            id: id.clone(),
            username,
            password,
        };

        let conn = self.pool.get().await?;
        conn.interact(move |conn| {
            diesel::insert_into(crate::schema::user::table)
                .values(&new_user)
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await??;

        Ok(id)
    }

    pub async fn sign_in(
        &self,
        input_username: String,
        input_password: String,
    ) -> Result<String, StoreError> {
        let conn = self.pool.get().await?;
        let user_result = conn
            .interact(move |conn| {
                use crate::schema::user::dsl::*;
                user.filter(username.eq(input_username))
                    .select(User::as_select())
                    .first(conn)
            })
            .await??;

        match crate::password::verify_password(&input_password, &user_result.password) {
            Ok(true) => Ok(user_result.id),
            Ok(false) | Err(_) => Err(StoreError::Diesel(DieselError::NotFound)),
        }
    }
}
