use super::schema::servers;

#[derive(Queryable, Debug)]
pub struct Server {
    pub id: i64,
    pub prefix: String,
    pub react: bool,
}

#[derive(Insertable)]
#[table_name="servers"]
pub struct NewServer<'a> {
    pub id: i64,
    pub prefix: &'a str,
    pub react: bool,
}