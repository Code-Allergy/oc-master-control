use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;

pub mod models;
pub mod schema;

pub type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConn = diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub fn establish_connection_pool(database_url: &str) -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
