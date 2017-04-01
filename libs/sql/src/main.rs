#[macro_use] extern crate sql_derive;

fn main() {
    println!("Hello, world!");
}

#[derive(SqlDataObject)]
pub struct Employee {
    pub id: u32,
    pub name: String
}