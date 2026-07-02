use sqlx::{
    Row, 
    query, 
};

use axum::{
    http::StatusCode,
    Json,
    extract::State,
};

use axum_extra::extract::cookie::{
    Cookie,
    CookieJar,
    SameSite,
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

use serde::{self, Deserialize , Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct LoginReq {
    uname: String,
    passwd: String,
}

#[derive(Deserialize, Debug)]
pub struct SiupReq {
    uname: String,
    passwd: String,
    email: String,
}

#[derive(Deserialize , Debug)]
pub struct ResetReq{
    umail : String
}
#[derive(Serialize , Debug)]
pub struct AppResponse<T:Serialize>{
    pub message : String,
    pub data : Option<T>
}

impl<T : Serialize> AppResponse<T> {
    pub fn ok(message : &str , data : T) -> Self{
        Self{
            message : message.to_string(),
            data : Some(data)
        }
    }
    pub fn err(message : &str ) -> Self{
        Self{
            message : message.to_string(),
            data : None
        }
    }
}

pub async fn login_handler(
    State(pool): State<sqlx::SqlitePool>,
    jar : CookieJar,
    Json(lreq): Json<LoginReq>
) -> (StatusCode , CookieJar, Json<AppResponse<()>>){
    
     
    let row = match query("SELECT id , passwd FROM users WHERE uname = ?;")
        .bind(&lreq.uname)
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(row)) => row,
            Ok(None) => {
                return (StatusCode::UNAUTHORIZED , jar , Json(AppResponse::err("Invalid Credentials")));
            }
            Err(error) => {
                println!("The db crashed : {}", error);
                return (StatusCode::INTERNAL_SERVER_ERROR , jar , Json(AppResponse::err(&error.to_string())));
            }
        };
    let db_pass: String = row.get("passwd");

    match verify_pass(&lreq.passwd , &db_pass){
        Ok(true)    => {
            let uid : i64 = row.get("id");
            let sid = Uuid::now_v7().to_string();
            let curr = time::Timestamp::now();
            let exp_dur = time::Duration::days(3);
            let exp = curr + exp_dur;
            if let Err(error) = query(
                "INSERT INTO sessions(uid, sid, created, expire )
                VALUES (?, ?, ?, ?);"
            ).bind(&uid)
             .bind(&sid)
             .bind(&curr.as_seconds())
             .bind(&exp.as_seconds())
             .execute(&pool).await{
                 return (StatusCode::INTERNAL_SERVER_ERROR , jar , Json(AppResponse::err(&error.to_string())));
            }

            let cookie = Cookie::build(("sid" , sid.to_string()))
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .max_age(exp_dur)
                .build();

            let jar = jar.add(cookie);

            return (StatusCode::OK , jar , Json(AppResponse::ok("login_successfull", ())));
            
        },
        Ok(false)   => {
            return (StatusCode::UNAUTHORIZED , jar , Json(AppResponse::err("Invalid Credentials")));
        }
        Err(error)  => {
            return (StatusCode::INTERNAL_SERVER_ERROR , jar , Json(AppResponse::err(&error.to_string())));
        }
    }
}

pub async fn signup_handler(
    State(pool): State<sqlx::SqlitePool>,
    Json(siupreq): Json<SiupReq>,
) -> (StatusCode , Json<AppResponse<()>>) {

    let unameq = query("SELECT id FROM users WHERE uname = ?;")
        .bind(&siupreq.uname)
        .fetch_optional(&pool)
        .await;

    if let Ok(Some(_)) = unameq {
        return (StatusCode::CONFLICT , Json(AppResponse::err("uname not unique")));
    } else if let Err(error) = unameq {
        return (StatusCode::INTERNAL_SERVER_ERROR , Json(AppResponse::err(&error.to_string())));
    }

    let emailq = query("SELECT id FROM users WHERE email = ?;")
        .bind(&siupreq.email)
        .fetch_optional(&pool)
        .await;

    if let Ok(Some(_)) = emailq {
        return (StatusCode::CONFLICT , Json(AppResponse::err("email not unique")));
    } else if let Err(error) = emailq {
        return (StatusCode::INTERNAL_SERVER_ERROR , Json(AppResponse::err(&error.to_string())));
    }

    let hpass : String;

    match make_pass_hash(&siupreq.passwd){
        Ok(e) => hpass = e,
        Err(error) => {
            return (StatusCode::INTERNAL_SERVER_ERROR,
               Json(AppResponse::err(&format!("Pass Hash failed : {}" , error)))
               );
        }
    }

    let insertq = query("INSERT INTO users(uname , passwd , email) VALUES (? , ? , ?)")
        .bind(&siupreq.uname)
        .bind(&hpass)
        .bind(&siupreq.email)
        .execute(&pool)
        .await;

    if let Err(error) = insertq {
        return (StatusCode::INTERNAL_SERVER_ERROR , Json(AppResponse::err(&error.to_string())));
    }
    // verify email id here with otp thing idk...
    // i guess idk yet but we will do this later stage of the project
    // now is the time to built that smtp ? idk what it 
    // is called but that thing

    (StatusCode::OK , Json(AppResponse::ok("Sign Up Successfull" , ())))
}

pub async fn reset_handler(
    State(pool): State<sqlx::SqlitePool>,
    Json(rreq): Json<ResetReq>,
) -> String{

    // println!("---------------------------");
    // println!("Reset Req:\nUsername / Email => \"{}\"\n" , rreq.umail);
    // println!("---------------------------");

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
