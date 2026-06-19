
use reqwest::{
    Client,
};
use serde:: {
    Deserialize,
    de::DeserializeOwned,
};
use serde_json::{
    json,
    Value,
};
use axum::{
    Json,
    extract::State,
};

use crate::db::{
    insert_ani_details,
    search_ani_title,
    insert_learned_title,
};

#[derive(Deserialize , Debug)]
#[allow(dead_code , unused_variables)]
pub struct Response{
    pub data : Data,
}

#[derive(Deserialize , Debug)]
#[allow(dead_code , unused_variables)]
pub struct Data{
    #[serde(rename = "Media")]
    pub media : AniDetails,
}

#[derive(Deserialize , Debug)]
#[allow(dead_code , unused_variables)]
pub struct AniDetails{
    pub id : i64,
    #[serde(rename = "episodes")]
    pub ep : Option<u32>,
    pub status : String,
    pub title : AniTitle,
    #[serde(rename = "description")]
    pub des : String,
    #[serde(rename = "bannerImage")]
    pub bimg : Option<String>,
    #[serde(rename = "coverImage")]
    pub cimg : CoverImage,
    pub genres : Vec<String>,
}


#[derive(Deserialize , Debug)]
#[allow(dead_code , unused_variables)]
pub struct AniTitle{
    pub english : Option<String>,
    pub romaji : Option<String>,
    pub native : Option<String>
}

#[derive(Deserialize , Debug)]
#[allow(dead_code , unused_variables)]
pub struct CoverImage{
    #[serde(rename = "extraLarge")]
    pub extra_large : String,
}

#[derive(Deserialize , Debug)]
pub struct Squerry{
    querry : String,
}


impl AniTitle{
    pub fn iter(&self) -> impl Iterator<Item = (&'static str , &str)>{
        [
            ("english", self.english.as_deref()),
            ("romaji",  self.romaji.as_deref()),
            ("native",  self.native.as_deref())
        ].into_iter()
        .filter_map(|(key , opt_val)| {
            opt_val.map(|val| (key , val))
        })

    }
}


const ANI_LIST_API  : &str = "https://graphql.anilist.co";

// TODO : all this functions

async fn fetch_anilist<T>(payload : &Value) -> Result<T , String> where T: DeserializeOwned{
    let client = Client::new();
    let response = match client
        .post(ANI_LIST_API)
        .json(payload)
        .send().await{
            Ok(res) => res,
            Err(error) => {
                return Err(format!("Response error : {}", error));
            }
        };
    match response.json::<T>().await{
        Ok(ele) => Ok(ele),
        Err(error) => Err(format!("Parse Error : {}", error)),
    }

}

#[allow(dead_code)]
pub async fn get_top_banners() -> String{
    "get_top_banners".to_string()
}
// TODO : all this functions
#[allow(dead_code)]
pub async fn get_trending() -> String{
    "get_trending".to_string()
}

// TODO : all this functions
#[allow(dead_code)]
pub async fn get_all_time() -> String{
    "get_all_time".to_string()
}

// TODO : all this functions
#[allow(dead_code)]
pub async fn get_current_season() -> String{
    "get_current_season".to_string()

}

// TODO : all this functions
#[allow(dead_code)]
pub async fn estimated_current_season_schedule() -> String{
    "estimated_current_season_schedule".to_string()
}

// TODO : all this functions
#[allow(dead_code , unused_variables)]
pub async fn get_all_season_for(Json(squerry) : Json<Squerry>) -> String{
    "get_all_season_for".to_string()
}

pub async fn ani_search(
    State(pool): State<sqlx::SqlitePool>,
    Json(squerry): Json<Squerry>
) -> Result<String , String> {

    let ani_detail; 
    let search =  search_ani_title(&pool , &squerry.querry).await;
    if let Ok(Some(id)) = search{
        println!("Got the thing here : {}" , id);
        return Ok(format!("Got the thing : {}" , id));
        // need to consturct anidetail here
        // but what and how will i architecture that
        // idk will i show single thing
        // or multiple things cause it is search
        // so...
    }else if let Ok(None) = search{
        println!("Fetch branch");
        let qry = r#"
        query ($search : String , $status: MediaStatus  ){
            Media (search : $search, type : ANIME , status : $status ) {
                id
                title{
                    romaji
                    english
                    native
                }
                bannerImage
                coverImage{
                    large
                    extraLarge
                }
                description
                episodes
                status
                genres
            }
        }
        "#;

        let payload = json!({
            "query" : qry,
            "variables" : {
                "search" : squerry.querry
            }
        });
        let response = match fetch_anilist::<Response>(&payload).await{
            Ok(ele) => ele,
            Err(error) => {
                return Err(format!("Error : {}", error))
            }
        };
        ani_detail = &response.data.media;
        if let Err(error) = insert_ani_details(ani_detail , &pool).await{

            if error.starts_with("Ani Error") && error.contains("1555"){
                if let Err(error) = insert_learned_title(&pool , &squerry.querry , &ani_detail.id).await{
                    return Err(format!("Error : {}", error));
                }
                return Ok(format!("Inserted new title : {}", &squerry.querry));

            }
            return Err(format!("Error : {}\nQuerry : '{}'", error , squerry.querry));
        }else{
            println!("Response : {:#?}",response);
            return Ok(format!("Good thing i guess"));
        };
    }else if let Err(error) = search{
        return Err(format!("Error : {}\nQuerry : '{}'", error , squerry.querry));
    }else{
        return Err(format!("Error : Reached Unreachable"));
    }

}
