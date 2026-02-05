use crate::store::Store;
use diesel::prelude::*;
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
    pub fn sign_up(&mut self, username:String, password:String) -> Result<String, diesel::result::Error>{
        let id = Uuid::new_v4();
        let u = User {
            username,
            password,
            id: id.to_string()
        };
        
        diesel::insert_into(crate::schema::user::table)
            .values(&u)
            .returning(User::as_returning())
            .get_result(&mut self.conn)?;

        Ok(id.to_string())
    }

    pub fn sign_in(&mut self, input_username:String, input_password:String) -> Result<String, diesel::result::Error>{
        use crate::schema::user::dsl::*;
        let user_result = user.filter(username.eq(input_username))
            .select(User::as_select())
            .first(&mut self.conn)?;
   
            match crate::password::verify_password(&input_password, &user_result.password) {
                Ok(true) => Ok(user_result.id),
                Ok(false) => Err(diesel::result::Error::NotFound),
                Err(_) => Err(diesel::result::Error::NotFound),  // Simplified - treat verification errors as not found
            }
    }
}