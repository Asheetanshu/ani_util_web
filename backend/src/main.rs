use sqlx::{
    query,
    sqlite::{SqlitePoolOptions},
    Row,
};

use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
};

use serde::{self, Deserialize};
use tower_http::cors::CorsLayer;

#[derive(Deserialize , Debug)]
struct LoginReq{
    uname : String,
    passwd : String,
}

#[derive(Deserialize , Debug)]
struct SiupReq{
    uname : String,
    passwd : String,
    email : String,
}


#[tokio::main]
async fn main(){
    println!("I guess working now");

    let pool = match SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:data/user.db").await{
            Ok(p) => p,
            Err(error) => {
                println!("The connection to the db cannot be made : {}", error);
                return;
            }
        };

    match query(
        "CREATE TABLE IF NOT EXISTS users(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uname varchar(16) NOT NULL UNIQUE,
            passwd varchar(32) NOT NULL,
            email varchar(128) NOT NULL UNIQUE
        ); "
    ).execute(&pool)
        .await{
            Ok(ele) => ele, 
            Err(error) => {
                println!("Well well something went wrong here : {}", error);
                return;
            }
        };
    let app = Router::new()
        .route("/login" , post(login_handler))
        .route("/signup" , post(signup_handler))
        .layer(CorsLayer::permissive())
        .with_state(pool);
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

async fn login_handler( State(pool) : State<sqlx::SqlitePool> , Json(lreq) :  Json<LoginReq>) -> String{
    println!("Req with\nUsername : {}\nPassword : {}\n",lreq.uname , lreq.passwd);
    match query(
        " SELECT passwd FROM users WHERE uname = ? "
    ).bind(&lreq.uname)
        .fetch_optional(&pool)
        .await{
            Ok(ele) =>{
                match ele {
                    Some(row) => {
                        let db_pass : String = row.get("passwd");
                        if db_pass == lreq.passwd {
                            "1".to_string()
                        }else{
                            "2".to_string()
                        }
                    },
                    None => {
                        "3".to_string()
                    },
                }
            },
            Err(error) => {
                println!("The db crashed : {}",error);
                error.to_string()
            },
        }
}


async fn signup_handler(State(pool) : State<sqlx::SqlitePool> , Json(siupreq) : Json<SiupReq>) -> String{
    println!("SingUp req with\nUsername : {}\nPassword : {}\nEmail : {}"
        , siupreq.uname 
        , siupreq.passwd 
        , siupreq.email
    );

    let unameq = query("SELECT id FROM users WHERE uname = ?;")
        .bind(&siupreq.uname)
        .fetch_optional(&pool)
        .await;

    if let Ok(Some(_)) = unameq{
        return "2".to_string();
    }else if let Err(error) = unameq{
        return error.to_string();
    }

    let emailq = query("SELECT id FROM users WHERE email = ?;")
        .bind(&siupreq.email)
        .fetch_optional(&pool)
        .await;

    if let Ok(Some(_)) = emailq{
        return "3".to_string();
    }else if let Err(error) = emailq{
        return error.to_string();
    }

    let insertq = query("INSERT INTO users(uname , passwd , email) VALUES (? , ? , ?)")
        .bind(&siupreq.uname)
        .bind(&siupreq.passwd)
        .bind(&siupreq.email)
        .execute(&pool)
        .await;

    if let Err(error) = insertq{
        return error.to_string();
    }
    // verify id here... i guess idk yet but we will do this later stage of the project
    "1".to_string()
}
