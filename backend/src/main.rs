mod auth;
mod ani_meta;
mod db;

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

use db::{
    init_pool,
    init_db,
};

use ani_meta::{
    ani_search,
};


#[tokio::main]
async fn main() {

    let pool = match init_pool("sqlite:data/main.db").await
    {
        Ok(ele) => ele,
        Err(error) => {
            println!("{}" , error);
            return;
        }
    };

    if let Err(error) = init_db(&pool).await{
        println!("{}" , error);
        return;
    };

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
        .route("/app/ani_search", post(ani_search))
        .fallback_service(ServeDir::new("../frontend"))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let port_thing = "127.0.0.1:6769";

//    let _ = ani_meta::get_api_response().await;

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
