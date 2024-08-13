use std::collections::HashMap;
use site::{database, routes, shutdown_signal, AppState};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

const SERVER_ADDR: &str = "0.0.0.0:3000";

/*
   ae_history {

   }

   power_history {
       current_power,
       max_power,
       current_draw,
       
   }

   .. other tables of history


   status enum { (text in db)
       Authorized
       Enrolled
       Connected
   }

   * with this design, we save everything to database, then wipe the Authorized (unenrolled) keys
   after some set period

*/

// primary todos:
//
// working errors for other types, and possibly able to pass in error code in the text,
// enum for main AppError, can have other type errors automatically translated and returned.

// websocket handler and command/control portion of website/client
//

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().expect("Failed to load .env"); // TODO TMP DEV-REMOVE-ME

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let state = Arc::new(AppState {
        pool: database::establish_connection_pool(&database_url),
        active_clients: Arc::new(Mutex::new(HashMap::new())),
    });
    
    let listener = tokio::net::TcpListener::bind(SERVER_ADDR).await.unwrap();

    println!("Serving at {SERVER_ADDR}!");
    axum::serve(listener, routes::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
