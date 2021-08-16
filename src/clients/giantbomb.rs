use actix_web::client::Client;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct GameImage {
  original_url: Option<String>,
  super_url: Option<String>,
  screen_url: Option<String>,
  screen_large_url: Option<String>,
  medium_url: Option<String>,
  small_url: Option<String>,
  thumb_url: Option<String>,
  icon_url: Option<String>,
  tiny_url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Characteristic {
  api_detail_url: String,
  id: i32,
  name: String,
  site_detail_url: String,
  abbreviation: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
  id: i32,
  guid: String,
  image: Option<GameImage>,
  name: String,
  deck: Option<String>,
  description: Option<String>,
  original_release_date: Option<String>,
  site_detail_url: Option<String>,
  expected_release_day: Option<String>,
  expected_release_month: Option<String>,
  expected_release_year: Option<String>,
  expected_release_quarter: Option<String>,
  platforms: Option<Vec<Characteristic>>,
  concepts: Option<Vec<Characteristic>>,
  developers: Option<Vec<Characteristic>>,
  characters: Option<Vec<Characteristic>>,
  themes: Option<Vec<Characteristic>>,
}

#[derive(Deserialize, Debug)]
struct DetailUrl {
  api_detail_url: String,
}

#[derive(Deserialize, Debug)]
struct GiantBombResponse {
  // I can't remember what exactly these values can be, but just someting that isn't OK will be enough
  // will be either "OK" | "ERROR"
  error: String,
  version: String,
  limit: i32,
  offset: i32,
  number_of_page_results: i64,
  number_of_total_results: i64,
  status_code: i32,
  results: Vec<DetailUrl>,
}

#[derive(Deserialize, Debug)]
struct GiantBombGameResponse {
  // I can't remember what exactly these values can be, but just someting that isn't OK will be enough
  // will be either "OK" | "ERROR"
  error: String,
  version: String,
  limit: i32,
  offset: i32,
  number_of_page_results: i64,
  number_of_total_results: i64,
  status_code: i32,
  results: Game,
}

#[derive(Deserialize, Debug)]
struct DatasResponse {
  status: String,
  result: Game,
}

fn random(max: i64) -> i64 {
  // get random int between 0 and (max - 1)
  rand::thread_rng().gen_range(0..max)
}

async fn get_max_games(client: &Client, token: &str) -> Result<i64, actix_web::Error> {
  let url = format!(
    "https://www.giantbomb.com/api/games/?api_key={}&limit=1&field_list=api_detail_url&format=json",
    token
  );
  let response = client
    .get(url)
    .set_header("Accept-Language", "application/json")
    .send()
    .await?
    .json::<GiantBombResponse>()
    .await
    .map_err(actix_web::Error::from)?;

  Ok(response.number_of_total_results)
}

async fn get_game_uri(client: &Client, token: &str, idx: i64) -> Result<String, actix_web::Error> {
  let url = format!(
    "https://www.giantbomb.com/api/games/?api_key={}&limit=1&format=json&offset={}&field_list=api_detail_url",
    token, idx
  );

  let response = client
    .get(url)
    .set_header("Accept-Language", "application/json")
    .send()
    .await?
    .json::<GiantBombResponse>()
    .await
    .map_err(actix_web::Error::from)?;

  let url = response.results.get(0).map(|detail| &detail.api_detail_url);
  match url {
    None => Err(actix_web::error::ErrorBadGateway(
      "Failed to get api detail url from giant bomb",
    )),
    Some(url) => Ok(url.to_string()),
  }
}

async fn get_game_details(
  client: &Client,
  token: &str,
  uri: &str,
) -> Result<Game, actix_web::Error> {
  let url = format!(
    "{}?api_key={}&format=json&field_list={}",
    uri,
    token,
    [
      "name",
      "site_detail_url",
      "themes",
      "platforms",
      "original_release_date",
      "image",
      "id",
      "guid",
      "expected_release_year",
      "expected_release_quarter",
      "expected_release_month",
      "expected_release_day",
      "developers",
      "deck",
      "description",
      "concepts",
      "characters",
    ]
    .join(",")
  );

  let response = client
    .get(url)
    .set_header("Accept-Language", "application/json")
    .send()
    .await?
    .json::<GiantBombGameResponse>()
    .await
    .map_err(actix_web::Error::from)?;

  Ok(response.results)
}

pub async fn get_random_game(token: &str) -> Result<Game, actix_web::Error> {
  let client = Client::default();
  let max_games = get_max_games(&client, token).await?;
  log::info!("got max games: {}", max_games);

  let idx = random(max_games);
  log::info!("querying for game idx: {}", idx);

  let game_uri = get_game_uri(&client, token, idx).await?;
  log::info!("querying for game uri: {}", game_uri);

  let game = get_game_details(&client, token, &game_uri).await?;
  Ok(game)
}
