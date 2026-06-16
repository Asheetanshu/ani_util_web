use sqlx::{
    Row, 
    query, 
    sqlite::SqlitePoolOptions,
};

use axum::{
    Json,
    Router,
    extract::State,
    routing::post
};

use tower_http::{
    cors::CorsLayer,
    services::{
        ServeDir,
        ServeFile,
    }
};

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        SaltString,
    },
    Argon2,
};

use serde::{self, Deserialize};

#[derive(Deserialize, Debug)]
struct LoginReq {
    uname: String,
    passwd: String,
}

#[derive(Deserialize, Debug)]
struct SiupReq {
    uname: String,
    passwd: String,
    email: String,
}

#[derive(Deserialize , Debug)]
struct ResetReq{
    umail : String
}

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

async fn login_handler(
    State(pool): State<sqlx::SqlitePool>,
    Json(lreq): Json<LoginReq>
) -> String {
    println!("---------------------------");
    println!("Login Req:\nUsername => \"{}\"\nPassword => \"{}\"" , lreq.uname, lreq.passwd);
    println!("---------------------------");
    let row = match query("SELECT passwd FROM users WHERE uname = ?;")
        .bind(&lreq.uname)
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(row)) => row,
            Ok(None) => return "3".to_string(),
            Err(error) => {
                println!("The db crashed : {}", error);
                return error.to_string();
            }
        };
    let db_pass: String = row.get("passwd");

    match verify_pass(&lreq.passwd , &db_pass){
        Ok(true) =>  return "1".to_string(),
        Ok(false) => return "2".to_string(),
        Err(error) => return error.to_string(),
    }
}

async fn signup_handler(
    State(pool): State<sqlx::SqlitePool>,
    Json(siupreq): Json<SiupReq>,
) -> String {
    println!("---------------------------");
    println!("SingUp req with\nUsername : \"{}\"\nPassword : \"{}\"\nEmail : \"{}\"", siupreq.uname, siupreq.passwd, siupreq.email);
    println!("---------------------------");

    let unameq = query("SELECT id FROM users WHERE uname = ?;")
        .bind(&siupreq.uname)
        .fetch_optional(&pool)
        .await;

    if let Ok(Some(_)) = unameq {
        return "2".to_string();
    } else if let Err(error) = unameq {
        return error.to_string();
    }

    let emailq = query("SELECT id FROM users WHERE email = ?;")
        .bind(&siupreq.email)
        .fetch_optional(&pool)
        .await;

    if let Ok(Some(_)) = emailq {
        return "3".to_string();
    } else if let Err(error) = emailq {
        return error.to_string();
    }

    let hpass : String;

    match make_pass_hash(&siupreq.passwd){
        Ok(e) => hpass = e,
        Err(error) => {
            return format!("Error : password hashing failed due to this : {}" , error);
        }
    }

    let insertq = query("INSERT INTO users(uname , passwd , email) VALUES (? , ? , ?)")
        .bind(&siupreq.uname)
        .bind(&hpass)
        .bind(&siupreq.email)
        .execute(&pool)
        .await;

    if let Err(error) = insertq {
        return error.to_string();
    }
    // verify email id here with otp thing idk...
    // i guess idk yet but we will do this later stage of the project
    // now is the time to built that smtp ? idk what it 
    // is called but that thing

    "1".to_string()
}

async fn reset_handler(
    State(pool): State<sqlx::SqlitePool>,
    Json(rreq): Json<ResetReq>,
) -> String{

    println!("---------------------------");
    println!("Reset Req:\nUsername / Email => \"{}\"\n" , rreq.umail);
    println!("---------------------------");

    let umailq = query("SELECT id FROM users WHERE uname = ? or email = ?;")
        .bind(&rreq.umail)
        .bind(&rreq.umail)
        .fetch_optional(&pool)
        .await;

    if let Ok(Some(_)) = umailq{
        return "1".to_string();
    }else if let Ok(None) = umailq{
        return "2".to_string();
    }else if let Err(ele) = umailq{
        return ele.to_string();
    } 

    "Something bad happened That shouldn't have happened".to_string()

        // now that they we know that they exist in my system 
        // now we send them that email which they will open 
        // and reconfirm their password and after that the js
        // will give another thing for us to check and then
        // we will change the password of that username
        // with the new one
}

fn make_hyprlink(text : &str , link : &str) -> String{
    format!(
        "\x1b]8;;http://{}\x1b\\{}\x1b]8;;\x1b\\",
        link,
        text
    )
}

fn make_pass_hash(ppass: &str) -> Result<String , String>{

    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    match argon2
        .hash_password(ppass.as_bytes() , &salt){
            Ok(ele) => return Ok(ele.to_string()),
            Err(error) => return Err(error.to_string())
        }
}
fn verify_pass(ppas : &str , dpas : &str) -> Result<bool , String>{
    let parsed_hash = match PasswordHash::new(dpas){
        Ok(ele) => ele,
        Err(error) => {
            let temp = format!("Failed to parse the db pass due to this : {} " , error);
            println!("{}" , temp);
            return Err(temp);
        }
    };

    let argon2 = Argon2::default();

    if let Err(error) = argon2.verify_password(ppas.as_bytes() , &parsed_hash){
        println!("this error i suppose : {}" , error);
        return Ok(false);
    }
    Ok(true)
}
