use sqlx::{
    query,
    sqlite::{SqliteConnectOptions, SqlitePool},
};
use axum::{
    routing::post,
    Router,
    Json,
};
use serde::{self, Deserialize};
use tower_http::cors::CorsLayer;

#[derive(Deserialize , Debug)]
struct LoginReq{
    uname : String,
    passwd : String,
}


#[tokio::main]
async fn main(){
    println!("I guess working now");
    let app = Router::new()
        .route("/login" , post(login_handler))
        .layer(CorsLayer::permissive());
    let listener  = match tokio::net::TcpListener::bind("127.0.0.1:6769").await{
        Ok(ele) => ele ,
        Err(error) => {
            println!("Please try again the stuff is broken i suppose. Error : {}" ,error);
            return;
        }
    };
    match axum::serve(listener , app).await{
        Ok(()) => (),
        Err(error) => {
            println!("This error : {}",error);
            return;
        },
    }
    return;
}

async fn login_handler(Json(lreq) :  Json<LoginReq>) -> String{
    println!("Req with\nUsername : {}\nPassword : {}\n",lreq.uname , lreq.passwd);
    "Yeah you reached the thing".to_string()
}
