mod data;
mod env;

use std::net::{IpAddr, SocketAddr};
use axum::{Router};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::get;

#[tokio::main]
async fn main() {
	let db = data::Database::new().await;
	let root_db = db.clone();
	let path_db = db.clone();

	let app = Router::new()
		.route("/", get(|| async move {
			let count = root_db.inc_and_get_count(&"")
				.await;

			let Ok(count) = count else {
				return (StatusCode::INTERNAL_SERVER_ERROR, "An error occurred".to_string())
			};

			let greeting = format!("Hello, World!\nI've greeted you {count} time(s).");

			(StatusCode::OK, greeting)
		}))
		.route("/:name", get(|Path(user_id): Path<String>| async move {
			let count = path_db.inc_and_get_count(&user_id)
				.await;

			let Ok(count) = count else {
				return (StatusCode::INTERNAL_SERVER_ERROR, "An error occurred".to_string());
			};

			let greeting = format!("Hello, {user_id}!\nI've greeted you {count} time(s).");

			(StatusCode::OK, greeting)
		}));


	let socket_addr = get_socket_addr();
	axum::Server::bind(&socket_addr)
		.serve(app.into_make_service())
		.await
		.unwrap();
}

fn get_socket_addr() -> SocketAddr {
	let port = get_port();
	let ip = IpAddr::from([0, 0, 0, 0]);
	SocketAddr::new(ip, port)
}

fn get_port() -> u16 {
	env::must_get("PORT")
		.parse()
		.expect("value in PORT environment expected to be a unsigned 16-bit integer")
}
