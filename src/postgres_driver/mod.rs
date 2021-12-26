mod conn;
pub mod driver;
mod errors;
mod message;
mod protocol_handler;
mod utils;

// how transactions are handled.
// since simple query can have mutiple statement. it's considered as single transaction but if there is a transaction block
// in simple query then it's split into multiple transaction for example.
// BEGIN;
// INSERT INTO mytable VALUES(1);
// COMMIT;
// INSERT INTO mytable VALUES(2);
// SELECT 1/0;
// first transction block should be committed and next should not run. So, we have to manually reffernce count of transaction block
// to handle statement safely. //TODO: do refernce count and handle the statement carefully.