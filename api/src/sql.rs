use rocket_contrib::databases::postgres;


pub trait SqlItem {
    // Parse item from postgres type
    fn from_row(row: &postgres::Row) -> Self;

    // Query all objects from database (parsed)
    fn get_all(c: &mut postgres::Client, namespace: String) -> Vec<Self> where Self: Sized;
    // Query certain amount of object from database (parsed)
    fn get_page(c: &mut postgres::Client, namespace: String, page: u32, page_size: u32) -> Vec<Self> where Self: Sized;

    // Insert item into database, returns id
    fn insert(&self, c: &mut postgres::Client) -> u32;
    // Put item into the database (create or update), returns id
    fn put(&self, c: &mut postgres::Client, id: u32) -> u32;

    // Delete all objects but first count them, returns amount of deleted items
    fn delete_all(c: &mut postgres::Client, namespace: String) -> u64;
}

