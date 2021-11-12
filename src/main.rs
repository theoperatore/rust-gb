mod clients;
mod error;

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use error::AppError;
use std::env;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

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
  let app_name = "gb-rs";
  dotenv().ok();

  // Redirect all "log"'s events to the subscriber
  LogTracer::init().expect("Failed to set log logger");
  let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  let formatting_layer = BunyanFormattingLayer::new(app_name.into(), std::io::stdout);

  let subscriber = Registry::default()
    .with(env_filter)
    .with(JsonStorageLayer)
    .with(formatting_layer);

  set_global_default(subscriber).expect("Failed to set subscriber");

  HttpServer::new(move || {
    let gb_token = env::var("GB_TOKEN").expect("GB_TOKEN env is required");
    App::new()
      .wrap(tracing_actix_web::TracingLogger)
      .wrap(middleware::Compress::default())
      .app_data(web::Data::new(AppContext { token: gb_token }))
      .service(ping)
      .service(get_random_game)
  })
  .bind("127.0.0.1:8080")?
  .run()
  .await?;

  Ok(())
}
