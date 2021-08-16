mod clients;
mod error;

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use error::AppError;

struct AppContext {
  token: String,
}

#[get("/_ping")]
async fn ping() -> impl Responder {
  HttpResponse::NoContent()
}

#[get("/game/random")]
async fn get_random_game(ctx: web::Data<AppContext>) -> impl Responder {
  clients::giantbomb::get_random_game(&ctx.token)
    .await
    .and_then(|response| Ok(HttpResponse::Ok().json(response)))
    .map_err(|err| AppError {
      msg: err.to_string(),
      status: 502,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info,info");
  env_logger::init();

  HttpServer::new(|| {
    let gb_token = std::env::var("GB_TOKEN").expect("GB_TOKEN");
    App::new()
      .wrap(middleware::Logger::default())
      .wrap(middleware::Compress::default())
      .data(AppContext { token: gb_token })
      .service(ping)
      .service(get_random_game)
  })
  .bind("127.0.0.1:8080")?
  .run()
  .await?;

  Ok(())
}
