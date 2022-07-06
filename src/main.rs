use std::fmt::Display;
use std::sync::Mutex;

use actix_files as fs;
use actix_web::{App, get, HttpResponse, HttpServer, middleware, post, Responder, ResponseError, Result, web};
use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Tweet {
  id: String,
  author: String,
  message: String,
  created_at: chrono::NaiveDateTime,
  likes: u8,
}

#[derive(Serialize, Deserialize)]
struct TweetRequest {
  author: String,
  message: String,
}

#[derive(Debug, Serialize)]
struct ErrNoId {
  id: String,
  err: String,
}

// Implement ResponseError for ErrNoId
impl ResponseError for ErrNoId {
  fn status_code(&self) -> StatusCode {
    StatusCode::NOT_FOUND
  }

  fn error_response(&self) -> HttpResponse<BoxBody> {
    let body = serde_json::to_string(&self).unwrap();
    let res = HttpResponse::new(self.status_code());
    res.set_body(BoxBody::new(body))
  }
}

// Implement Display for ErrNoId
impl Display for ErrNoId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}


pub struct AppState {
  tweets: Mutex<Vec<Tweet>>,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let app_state = web::Data::new(AppState {
    tweets: Mutex::new(vec![
      Tweet {
        id: Uuid::new_v4().to_string(),
        author: String::from("zig"),
        message: String::from("Hello, world!"),
        created_at: chrono::Utc::now().naive_utc(),
        likes: 0,
      },
      Tweet {
        id: Uuid::new_v4().to_string(),
        author: String::from("qwe"),
        message: String::from("Hi !"),
        created_at: chrono::Utc::now().naive_utc(),
        likes: 0,
      },
    ])
  });

  HttpServer::new(move || {
    App::new()
      .app_data(app_state.clone())
      .wrap(middleware::Logger::default())
      .configure(app_config)
  })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn app_config(config: &mut web::ServiceConfig) {
  config.service(
    web::scope("")
      .service(web::resource("/").route(web::get().to(index)))
      .service(fs::Files::new("/static", "static").show_files_listing())
      .service(handle_tweets)
      .service(post_tweet)
      .service(like_tweet));
}

async fn index() -> Result<HttpResponse> {
  Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(include_str!("../static/index.html")))
}

#[get("/api/tweets")]
pub async fn handle_tweets(data: web::Data<AppState>) -> impl Responder {
  let tweets = data.tweets.lock().unwrap();

  let response = serde_json::to_string(&(*tweets)).unwrap();

  HttpResponse::Ok()
    .content_type(ContentType::json())
    .body(response)
}

#[post("/api/tweets")]
async fn post_tweet(req: web::Json<TweetRequest>, data: web::Data<AppState>) -> impl Responder {
  let new_tweet = Tweet {
    id: Uuid::new_v4().to_string(),
    author: String::from(&req.author),
    message: String::from(&req.message),
    created_at: chrono::Utc::now().naive_utc(),
    likes: 0,
  };

  let mut tweets = data.tweets.lock().unwrap();
  let response = serde_json::to_string(&new_tweet).unwrap();
  tweets.push(new_tweet);

  HttpResponse::Created()
    .content_type(ContentType::json())
    .body(response)
}

#[post("/api/tweets/{id}/like-tweet")]
async fn like_tweet(id: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
  let tweet_id: String = id.to_string();
  let mut tweets = data.tweets.lock().unwrap();
  let id_index = tweets.iter()
    .position(|x| x.id == tweet_id);

  match id_index {
    Some(id) => {
      tweets[id].likes += 1;
      let response = serde_json::to_string(&tweets[id]).unwrap();
      Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response)
      )
    }
    None => {
      let response = ErrNoId {
        id: tweet_id,
        err: String::from("Tweet not found"),
      };
      Err(response)
    }
  }
}