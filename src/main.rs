use std::net::{IpAddr, SocketAddr};
use axum::{Router};
use axum::extract::Path;
use axum::routing::get;

#[tokio::main]
async fn main() {
	let app = Router::new()
		.route("/", get(|| async { "Hello, World!" }))
		.route("/:name", get(|Path(user_id): Path<String>| async move { format!("Hello, {user_id}") }));


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
	env::get("PORT")
		.parse()
		.expect("value in PORT environment expected to be a unsigned 16-bit integer")
}

mod env {
	use std::ffi::OsStr;
	use std::fmt::Display;

	pub fn get<K: AsRef<OsStr> + Display + Copy>(key: K) -> String {
		match std::env::var(key) {
			Ok(val) => val,
			Err(std::env::VarError::NotPresent) => panic!("expected {key} to be present in environment"),
			Err(std::env::VarError::NotUnicode(_)) => panic!("expected value for key {key} in environment to be valid UTF-8"),
		}
	}
}
