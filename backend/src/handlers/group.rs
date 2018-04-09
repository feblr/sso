use rocket::State;
use rocket_contrib::Json;

use super::super::models::group;
use super::super::models::group::Group;
use super::super::storage::Database;
use super::Error;

#[get("/groups")]
fn select_groups(db: State<Database>) -> Result<Json<Vec<Group>>, Error> {
    let conn = db.get_conn()?;
    let groups = group::select(&*conn)?;

    Ok(Json(groups))
}
