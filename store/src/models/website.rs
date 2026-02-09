use crate::store::Store;
use chrono::Utc;
use diesel::{Insertable, prelude::*};
use uuid::Uuid;
#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::website)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Website {

    pub id: String,
    pub url: String,
    pub user_id: String,
    pub time_added: chrono::NaiveDateTime
}


impl Store {
    pub fn create_website(&mut self, user_id: String, url: String) -> Result<Website, diesel::result::Error> {
        let id = Uuid::new_v4();
        let website = Website{
            user_id,
            url,
            id:id.to_string(),
            time_added: Utc::now().naive_utc()
        };

        diesel::insert_into(crate::schema::website::table)
        .values(&website)
        .returning(Website::as_returning())
        .get_result(&mut self.conn)?;

    Ok(website)
    
    }
    pub fn get_website(&mut self, input_id:String) -> Result<Website, diesel::result::Error>{
        use crate::schema::website::dsl::*;
        let website_result= website.filter(id.eq(input_id))
        .select(Website::as_select())
        .first(&mut self.conn)?;
    Ok(website_result)
    }

    pub fn list_websites(&mut self, input_user_id: String) -> Result<Vec<Website>, diesel::result::Error>{
        use crate::schema::website::dsl::*;
        let websites = website.filter
        (user_id.eq(input_user_id.clone()))
        .order(time_added.desc())
        .select(Website::as_select())
        .load(&mut self.conn)?;
    Ok(websites)  
    }

    pub fn update_website(
        &mut self,
        website_id: String,
        input_user_id: String,
        new_url: String
    ) -> Result<Website, diesel::result::Error> {
        use crate::schema::website::dsl::*;
        let updated = diesel::update(
            website
        ).filter(id.eq(website_id.clone()))
        .filter(user_id.eq(input_user_id.clone()))
        .set(url.eq(new_url))
        .returning(Website::as_returning())
        .get_result(&mut self.conn)?;
    Ok(updated)
    }

    pub fn delete_website (
        &mut self,
        website_id: String,
        input_user_id :String 
    ) -> Result< usize, diesel::result::Error> {
        use crate::schema::website::dsl::*;
        let deleted = diesel::delete(website)
        .filter(id.eq(website_id.clone()))
        .filter(user_id.eq(input_user_id.clone()))
        .execute(&mut self.conn)?;
    if deleted == 0 {
        return Err(diesel::result::Error::NotFound);
    }
    Ok(deleted)
}
}