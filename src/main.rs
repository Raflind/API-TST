use tide::{Request, Response, StatusCode};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    status: String,
    message: String,
}

#[derive(Deserialize)]
struct LogoutRequest {
    username: String,
}

#[derive(Serialize)]
struct LogoutResponse {
    status: String,
    message: String,
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    status: String,
    message: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct Movie {
    id: i32,
    adult: bool,
    backdrop_path: Option<String>,
    genre_ids: String,
    origin_country: String,
    original_language: Option<String>,
    original_name: Option<String>,
    original_title: Option<String>,
    overview: Option<String>,
    popularity: f64,
    poster_path: Option<String>,
    first_air_date: Option<String>,
    release_date: Option<String>,
    name: Option<String>,
    title: Option<String>,
    video: bool,
    vote_average: f64,
    vote_count: i32,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let pool = SqlitePool::connect("sqlite:./movies.db").await?;

     sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL,
            logged_in BOOLEAN NOT NULL DEFAULT 0
        )
        "#
    )
    .execute(&pool)
    .await?;

    let mut app = tide::with_state(pool);
    app.at("/movies").get(get_movies);

    app.at("/register").post(register);

    app.at("/login").post(login);

    app.at("/logout").post(logout);

    println!("Server running at http://0.0.0.0:8080");

    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

async fn login(mut req: Request<SqlitePool>) -> tide::Result {
    let data: LoginRequest = req.body_json().await?;
    let pool = req.state();

    let row = sqlx::query(
        r#"
        SELECT id, password
        FROM users 
        WHERE username = ?
        "#
    )
    .bind(&data.username)
    .fetch_optional(pool)
    .await?;

    if row.is_none() {
        let mut res = Response::new(StatusCode::Unauthorized);
        res.set_body(tide::Body::from_json(&LoginResponse {
            status: "Failed".into(),
            message: "User tidak ditemukan".into(),
        })?);
        return Ok(res);
    }

    let row = row.unwrap();

    let user_id: i64 = row.try_get("id")?;
    let db_password: String = row.try_get("password")?;

    if !bcrypt::verify(&data.password, &db_password)? {
        let mut res = Response::new(StatusCode::Unauthorized);
        res.set_body(tide::Body::from_json(&LoginResponse {
            status: "Failed".into(),
            message: "Password salah".into(),
        })?);
        return Ok(res);
    }

    sqlx::query(
        "UPDATE users SET logged_in = 1 WHERE id = ?"
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(tide::Body::from_json(&LoginResponse {
        status: "success".into(),
        message: "Berhasil Login".into(),
    })?);

    Ok(res)
}


async fn logout(mut req: Request<SqlitePool>) -> tide::Result {
    let data: LogoutRequest = req.body_json().await?;
    let pool = req.state();

    let result = sqlx::query(
        "UPDATE users SET logged_in = 0 WHERE username = ?"
    )
    .bind(&data.username)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        let mut res = Response::new(StatusCode::BadRequest);
        res.set_body(tide::Body::from_json(&LogoutResponse {
            status: "error".into(),
            message: "User tidak ditemukan".into(),
        })?);
        return Ok(res);
    }

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(tide::Body::from_json(&LogoutResponse {
        status: "success".into(),
        message: "Sayonara".into(),
    })?);

    Ok(res)
}


async fn register(mut req: Request<SqlitePool>) -> tide::Result {
    let data: RegisterRequest = req.body_json().await?;

    let pool = req.state();

    let hashed = bcrypt::hash(data.password, bcrypt::DEFAULT_COST)?;

    let result = sqlx::query(
        r#"
        INSERT INTO users (username, password, logged_in)
        VALUES (?, ?, 0)
        "#
    )
    .bind(data.username)
    .bind(hashed)
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            let mut res = Response::new(StatusCode::Ok);
            res.set_body(tide::Body::from_json(&RegisterResponse {
                status: "success".into(),
                message: "Berhasil Daftar".into(),
            })?);
            Ok(res)
        }
        Err(e) => {
            let mut res = Response::new(StatusCode::BadRequest);
            res.set_body(tide::Body::from_json(&RegisterResponse {
                status: "Failed".into(),
                message: format!("Gagal Daftar: {}", e),
            })?);
            Ok(res)
        }
    }
}

async fn get_movies(req: Request<SqlitePool>) -> tide::Result {
    let pool = req.state();

    let movies: Vec<Movie> = sqlx::query_as::<_, Movie>(
        r#"
        SELECT 
            id, adult, backdrop_path, genre_ids, origin_country,
            original_language, original_name, original_title,
            overview, popularity, poster_path, first_air_date,
            release_date, name, title, video, vote_average, vote_count
        FROM movies
        "#
    )
    .fetch_all(pool)
    .await?;

    let base_url = "https://image.tmdb.org/t/p/original";
    let movies: Vec<Movie> = movies.into_iter().map(|mut movie| {
        if let Some(ref path) = movie.backdrop_path {
            movie.backdrop_path = Some(format!("{}{}", base_url, path));
        }
        if let Some(ref path) = movie.poster_path {
            movie.poster_path = Some(format!("{}{}", base_url, path));
        }
        movie
    }).collect();

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(tide::Body::from_json(&movies)?);
    Ok(res)
}
