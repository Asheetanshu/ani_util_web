use sqlx::{
    Row, 
    query, 
};

use axum::{
    Json,
    extract::State,
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
pub async fn login_handler(
    State(pool): State<sqlx::SqlitePool>,
    Json(lreq): Json<LoginReq>
) -> String {
    
    // println!("---------------------------");
    // println!("Login Req:\nUsername => \"{}\"\nPassword => \"{}\"" , lreq.uname, lreq.passwd);
    // println!("---------------------------");
     
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

pub async fn signup_handler(
    State(pool): State<sqlx::SqlitePool>,
    Json(siupreq): Json<SiupReq>,
) -> String {
    // println!("---------------------------");
    // println!("SingUp req with\nUsername : \"{}\"\nPassword : \"{}\"\nEmail : \"{}\"", siupreq.uname, siupreq.passwd, siupreq.email);
    // println!("---------------------------");

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
