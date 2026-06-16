mod auth;
use sqlx::{
    query, 
    sqlite::SqlitePoolOptions,
};
use axum::{
    Router,
    routing::post
};
use tower_http::{
    cors::CorsLayer,
    services::{
        ServeDir,
        ServeFile,
    }
};
use auth::{
    login_handler,
    signup_handler,
    reset_handler,
};


#[tokio::main]
async fn main() {

    let pool = match SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:data/user.db")
        .await
        {
            Ok(p) => p,
            Err(error) => {
                println!("The connection to the db cannot be made : {}", error);
                return;
            }
        };

    if let Err(error) = query(
        "CREATE TABLE IF NOT EXISTS users(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uname varchar(16) NOT NULL UNIQUE,
            passwd varchar(256) NOT NULL DEFAULT 0,
            email varchar(128) NOT NULL UNIQUE
        ); ",
    )
        .execute(&pool)
            .await{
                println!("Well well something went wrong here : {}", error);
                return;
    }

    if let Err(error) = query(
        "CREATE TABLE IF NOT EXISTS passwd_reset(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email varchar(128) NOT NULL UNIQUE,
            attempts INTEGER CHECK (attempts IN (0, 1, 2, 3, 4, 5)) DEFAULT 0,
            expiry DATETIME NOT NULL,
            otp varchar(6) NOT NULL
        );")
        .execute(&pool)
            .await{
                println!("passwd_reset table not available here : {}", error);
                return;
    }


    let app = Router::new()
        .route_service("/" , ServeFile::new("../frontend/pages/index.html"))
        .route_service("/home" , ServeFile::new("../frontend/pages/index.html"))
        .route_service("/index" , ServeFile::new("../frontend/pages/index.html"))
        .route_service("/login" , ServeFile::new("../frontend/pages/login.html"))
        .route_service("/signup" , ServeFile::new("../frontend/pages/signup.html"))
        .route_service("/reset" , ServeFile::new("../frontend/pages/reset.html"))
        .route("/app/login", post(login_handler))
        .route("/app/signup", post(signup_handler))
        .route("/app/reset", post(reset_handler))
        .fallback_service(ServeDir::new("../frontend"))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let port_thing = "127.0.0.1:6769";

    let listener = match tokio::net::TcpListener::bind(port_thing).await {
        Ok(ele) => ele,
        Err(error) => {
            println!(
                "Please try again the stuff is broken i suppose. Error : {}",
                error
            );
            return;
        }
    };

    let link = make_hyprlink("AniWeb" , port_thing);

    println!("Server Running : {}",link);

    if let Err(error) = axum::serve(listener, app).await {
        println!("This error : {}", error);
        return;
    }
    return;
}

fn make_hyprlink(text : &str , link : &str) -> String{
    format!(
        "\x1b]8;;http://{}\x1b\\{}\x1b]8;;\x1b\\",
        link,
        text
    )
}

