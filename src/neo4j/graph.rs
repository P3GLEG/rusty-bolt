use std::collections::HashMap;

use neo4j::bolt::{BoltStream, BoltDetail, BoltSummary, BoltResponse};
use neo4j::packstream::Value;

#[macro_export]
macro_rules! parameters(
    {} => {
        {
            use std::collections::HashMap;

            HashMap::new()
        }
    };

    { $($key:expr => $value:expr),* } => {
        {
            use std::collections::HashMap;
            use $crate::neo4j::packstream::{Value, ValueCast};

            let mut map : HashMap<&str, Value> = HashMap::new();
            $(
                map.insert($key, ValueCast::from(&$value));
            )+;

            map
        }
    };
);

struct DummyResponse;
impl BoltResponse for DummyResponse {
    fn on_detail(&self, _: BoltDetail) {
        //
    }

    fn on_summary(&mut self, _: BoltSummary) {
        //
    }
}

struct InitResponse{
    summary: BoltSummary,
}
impl BoltResponse for InitResponse {
    fn on_detail(&self, _: BoltDetail) {
        panic!("Unexpected detail")
    }

    fn on_summary(&mut self, summary: BoltSummary) {
        self.summary = summary;
    }
}


// GRAPH //

pub struct Graph {}
impl Graph {
    pub fn new(address: &str, user: &str, password: &str) -> Box<GraphConnector> {
        Box::new(DirectBoltConnector::new(address, user, password))
    }
}


// GRAPH CONNECTOR //

pub trait GraphConnector {
    fn connect(&self) -> Box<GraphConnection>;
}
struct DirectBoltConnector {
    address: String,
    user: String,
    password: String,
}
impl DirectBoltConnector {
    pub fn new(address: &str, user: &str, password: &str) -> DirectBoltConnector {
        DirectBoltConnector { address: String::from(address),
                       user: String::from(user), password: String::from(password) }
    }
}
impl GraphConnector for DirectBoltConnector {
    fn connect(&self) -> Box<GraphConnection> {
        Box::new(DirectBoltConnection::new(&self.address[..], &self.user[..], &self.password[..]))
    }
}


// GRAPH CONNECTION //

pub trait GraphConnection {
    fn begin(&mut self);
    fn commit(&mut self);
    fn reset(&mut self);
    fn rollback(&mut self);
    fn run(&mut self, statement: &str, parameters: HashMap<&str, Value>);
    fn sync(&mut self);
}
struct DirectBoltConnection {
    connection: BoltStream,
}
impl DirectBoltConnection {
    pub fn new(address: &str, user: &str, password: &str) -> DirectBoltConnection {
        let mut connection = BoltStream::connect(address);
        let response = DummyResponse {};
        connection.pack_init(user, password, response);
        connection.sync();

        DirectBoltConnection { connection: connection, }
    }
}
impl GraphConnection for DirectBoltConnection {

    fn begin(&mut self) {
        self.connection.pack_run("BEGIN", parameters!(), DummyResponse {});
        self.connection.pack_discard_all(DummyResponse {});
    }

    fn commit(&mut self) {
        self.connection.pack_run("COMMIT", parameters!(), DummyResponse {});
        self.connection.pack_discard_all(DummyResponse {});
        self.connection.sync();
    }

    fn reset(&mut self) {
        self.connection.pack_reset(DummyResponse {});
        self.connection.sync();
    }

    fn rollback(&mut self) {
        self.connection.pack_run("ROLLBACK", parameters!(), DummyResponse {});
        self.connection.pack_discard_all(DummyResponse {});
        self.connection.sync();
    }

    fn run(&mut self, statement: &str, parameters: HashMap<&str, Value>) {
        self.connection.pack_run(statement, parameters, DummyResponse {});
        self.connection.pack_pull_all(DummyResponse {});
    }

    fn sync(&mut self) {
        self.connection.sync();
    }
}

#[cfg(test)]
mod test {

}