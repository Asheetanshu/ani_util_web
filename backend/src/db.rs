use sqlx::{
    SqlitePool, 
    query,
    query_scalar,
    Row,
    sqlite::SqlitePoolOptions
};

use crate::ani_meta::AniDetails;

pub async fn init_pool(path : &str) -> Result<SqlitePool , String>{
    let pool = match SqlitePoolOptions::new()
        .max_connections(5)
        .connect(path)
        .await{
            Ok(ele) => ele,
            Err(error) => {
                return Err(format!("Error : {}" , error));
            }

        };
    if let Err(error) = query("PRAGMA foreign_keys = ON;")
        .execute(&pool).await{
            return Err(format!("Error : {}" , error));

        };

    return Ok(pool);
}

pub async fn init_db(pool : &SqlitePool) -> Result<() , String>{
    if let Err(error) = query(
        "CREATE TABLE IF NOT EXISTS users(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uname varchar(16) NOT NULL UNIQUE,
            passwd varchar(256) NOT NULL DEFAULT 0,
            email varchar(128) NOT NULL UNIQUE
        ); "
    )
        .execute(pool)
            .await{
                return Err(format!("Error : {}", error));
    }

    if let Err(error) = query(
        "CREATE TABLE IF NOT EXISTS passwd_reset(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email varchar(128) NOT NULL UNIQUE,
            attempts INTEGER CHECK (attempts IN (0, 1, 2, 3, 4, 5)) DEFAULT 0,
            expiry DATETIME NOT NULL,
            otp varchar(6) NOT NULL
        );")
        .execute(pool).await{
            return Err(format!("Error : {}", error));
    }

    if let Err(error) = query(
        // 1) bImage and cImage are links
        // 2) staus is (FINISHED , RELEASING , NOT_YET_RELEASED) one of 
        //      these three which is provided by anilist api
        // 3) eps is total no. of episodes for that anime which is 
        //      released / yet to release 
        // 4) dps = description
        // 5) id will be provided by anilist api ( i think )
        "CREATE TABLE IF NOT EXISTS ani(    
            id INTEGER PRIMARY KEY,
            bImage TEXT ,               
            cImage TEXT NOT NULL,
            status VARCHAR(18) NOT NULL,
            eps INTEGER DEFAULT NULL,
            des TEXT NOT NULL,
            lst_updt DATETIME DEFAULT CURRENT_TIMESTAMP
        );")
        .execute(pool).await{
            return Err(format!("Error : {}" , error));
    }

    if let Err(error) = query(
"CREATE TABLE IF NOT EXISTS titles(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ttype VARCHAR(10) NOT NULL,
            title TEXT NOT NULL,
            aid INTEGER NOT NULL,
            FOREIGN KEY(aid) REFERENCES ani(id)
        );")
        .execute(pool).await{
            return Err(format!("Error : {}" , error));
    }

    if let Err(error) = query(
        "CREATE TABLE IF NOT EXISTS genres(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            gname VARCHAR(30) NOT NULL UNIQUE

        )")
        .execute(pool).await{
            return Err(format!("Error : {}" , error));
    }

    if let Err(error) = query(
        "CREATE TABLE IF NOT EXISTS anigen(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            gid INTEGER NOT NULL,
            aid INTEGER NOT NULL,
            FOREIGN KEY(gid) REFERENCES genres(id),
            FOREIGN KEY(aid) REFERENCES ani(id)
        )")
        .execute(pool).await{
            return Err(format!("Error : {}" , error));
    }

    if let Err(error) = query(
        // we could probablue use the browser to convert this time to 
        // the users local time zone and should also mention this
        // like for example : air_at : 08:40pm tuesday your local time : GMT + 5 : 30 
        "CREATE TABLE IF NOT EXISTS schedule(
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            aid INTEGER NOT NULL UNIQUE,
            nxt_ep INTEGER NOT NULL,
            air_at INTEGER NOT NULL
        );")
        .execute(pool).await{
            return Err(format!("Error : {}" , error));
    }
    Ok(())
}


pub async fn search_ani_title(pool : &SqlitePool , title : &str) -> Result<Option<i64> , String>{

    let sq = format!("%{}%",&title.trim().replace(" ","%"));
    let _ = match query_scalar("SELECT aid FROM titles WHERE title LIKE ?;")
        .bind(&sq)
        .fetch_optional(pool)
        .await{
            Ok(ele) => return Ok(ele),
            Err(error) => return Err(format!("Error : {}" , error)),

    };
}

pub async fn insert_ani_details(ani_details : &AniDetails , pool : &SqlitePool) -> Result<(), String>{
    let mut tx = match pool.begin().await{
        Ok(ele) => ele,
        Err(error) => return Err(format!("Failed : {}", error)),
    };
    if let Err(error) = query("INSERT INTO ani (id , status , eps , des , bImage , cImage ) VALUES (? , ? , ? , ? , ? , ? );")
        .bind(&ani_details.id)
        .bind(&ani_details.status)
        .bind(&ani_details.ep)
        .bind(&ani_details.des)
        .bind(&ani_details.bimg)
        .bind(&ani_details.cimg.extra_large)
        .execute(&mut *tx)
        .await{
            return Err(format!("Ani Error : {}", error));
            
        }
    for (ttype , ttitle) in ani_details.title.iter(){
        if let Err(error) = query("INSERT INTO titles(ttype , title , aid) VALUES (? , ? , ?);")
            .bind(&ttype)
            .bind(&ttitle)
            .bind(&ani_details.id)
            .execute(&mut *tx)
            .await{
                return Err(format!("Titles Error : {}", error));
        }
    }
    for g in ani_details.genres.iter(){
        if let Err(error) = query("INSERT or IGNORE INTO genres(gname) VALUES (?);")
            .bind(&g)
            .execute(&mut *tx)
            .await{
                return Err(format!("Genres Error : {}", error));
        }
        let gid : i64 = match query("SELECT id FROM genres WHERE gname = ?").bind(&g)
            .fetch_one(&mut *tx)
            .await
        {
            Ok(ele) => ele.get("id"), 
            Err(error) => return Err(format!("Gid Error : {}", error)),
        };
        if let Err(error) = query("INSERT INTO anigen(gid , aid) VALUES(? , ?)")
            .bind(&gid)
            .bind(&ani_details.id)
            .execute(&mut *tx)
            .await{
                return Err(format!("AniGen Error : {}", error));

        }
    }
    if let Err(error) = tx.commit().await{
        return Err(format!("Trans Commit Error : {}" , error));
    }
    Ok(())
}

pub async fn insert_learned_title(pool : &SqlitePool, title : &str , aid : &i64) -> Result<() , String>{
    match query("INSERT INTO titles(aid , ttype , title) VALUES(? , ? , ?);")
        .bind(aid)
        .bind("learned")
        .bind(&title)
        .execute(pool)
        .await{
            Ok(ele) => println!("{:#?}" , ele),
            Err(error) => {
                return Err(format!("Error : {}", error));
            }

        }

    Ok(())
}
