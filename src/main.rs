#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use schema::word_table;   
use schema::word_table::dsl::*;

use std::{env, fmt, error};

const DATABASE_URL: &str = "database.db";

#[derive(Debug)]
pub enum CustomError {
    AddError(String, String),
    EditError(String, String),
}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::AddError(word_val, count_val)    => write!(f, "AddError: Error adding {}: {} to table", word_val, count_val),
            CustomError::EditError(word_val, count_val)   => write!(f, "EditError: Error editing {}: {} to table", word_val, count_val),
        }
    }
}
impl error::Error for CustomError {}



//holds the info ont he table
pub mod schema;
//holds the struct version of our value
#[derive(Queryable, Debug, Clone)] //can be read from db
pub struct Table {
    pub word: String,
    pub count: i32,
}

//holds a value to write into db
#[derive(Insertable)]           //Can insert objects into db
#[table_name="word_table"]      //where it will be inserted
pub struct NewTable<'a> {
    pub word: &'a str,
    pub count: &'a i32,
}



//adds a word and count to db
pub fn add_word<'a>(word_val: &'a str, count_val: &'a i32) -> Result<Table, Box<dyn error::Error>>{
    let connection = SqliteConnection::establish(DATABASE_URL)?;
    let new_word = NewTable {                               //construct db entry
        word: word_val,
        count: count_val,
    };
    diesel::insert_into(word_table::table)                       //insert into the table
        .values(&new_word)                                  //the word
        .execute(&connection)?;                             //into the db
    let results = word_table::table                              //pull table to double check val was added
        .filter(word.like(format!("%{}%", new_word.word)))  //filter to find the word entry
        .load::<Table>(&connection)?;                       //from db
    if results.len() == 1 {Ok(results[0].clone())}
    else{Err(Box::new(CustomError::AddError(word_val.to_string(), count_val.to_string())))}
}



//search for word in db
fn search_word(word_val: &str) -> Result<Option<Table>, Box<dyn error::Error>>{
    let connection = SqliteConnection::establish(DATABASE_URL)?;
    let results = word_table.limit(5)
        .load::<Table>(&connection)?;
    for result in results {
        if result.word == word_val {
            return Ok(Some(result))
        }
    }
    Ok(None)
}



//dumps [n] word_table from db
fn dump_word_table(n: i64) -> Result<Vec<Table>, Box<dyn error::Error>>{
    let connection = SqliteConnection::establish(DATABASE_URL)?;
    let results = word_table.limit(n).load::<Table>(&connection)?;
    Ok(results)
}



fn edit_word(word_val: &str, count_val: i32) -> Result<(), Box<dyn error::Error>>{
    let connection = SqliteConnection::establish(DATABASE_URL)?;
    let _ = diesel::update(word_table.find(&word_val))
        .set(count.eq(count_val))
        .execute(&connection)?;
    let results = word_table::table                              //pull table to double check val was added
        .filter(word.like(format!("%{}%", word_val)))       //filter to find the word entry
        .load::<Table>(&connection)?;                       //from db
    if results.len() == 1 {if results[0].count==count_val {return Ok(())}}
    Err(Box::new(CustomError::AddError(word_val.to_string(), count_val.to_string())))
}



fn del_word(word_val: &str) -> Result<(), Box<dyn error::Error>>{
    let connection = SqliteConnection::establish(DATABASE_URL)?;
    let pattern = format!("%{}%", word_val);
    diesel::delete(word_table.filter(word.like(pattern)))
        .execute(&connection)?;
    Ok(())
}


fn main() {
    fn execute() -> Result<(), Box<dyn error::Error>> {
        let action = env::args().nth(1).expect("Expected an action");
        if action == "add" {
            let word_val = env::args().nth(2).expect("Expected word value");
            let count_val = env::args().nth(3).expect("Expected word count").parse()?;
            let result = add_word(&word_val, &count_val)?;
            println!("{:?}",result);
        }
        else if action == "dump" {
            let dump_count = env::args().nth(2).expect("Expected dump val").parse()?;
            for item in dump_word_table(dump_count)? {println!("{:?}", item)}
        }
        else if action == "search" {
            let word_val = env::args().nth(2).expect("Expected word value");
            let result = search_word(&word_val)?;
            println!("{:?}", result);
        }
        else if action == "edit" {
            let word_val = env::args().nth(2).expect("Expected word value");
            let count_val = env::args().nth(3).expect("Expected word count").parse()?;
            edit_word(&word_val, count_val)?;
        }
        else if action == "del" {
            let word_val = env::args().nth(2).expect("Expected word value");
            del_word(&word_val)?;
        }
        else{println!("Invalid action")}
        Ok(())
    }

    execute().unwrap_or_else(|e|panic!("{}",e))
}