use reqwest::{
    Client,
};
use serde:: {
    Deserialize,
    Serialize,
    de::DeserializeOwned,
};
use serde_json::{
    json,
    Value,
};
use axum::{
    Json,
    http::StatusCode,
    extract::State,
};
use crate::{
    auth::AppResponse,
    db::{
        insert_ani_details,
        search_ani_title,
        fetch_db_by_id
    }
};
use futures::stream::{
    FuturesUnordered, StreamExt
};

#[derive(Deserialize , Debug)]
#[allow(dead_code)]
pub struct Response{
    pub data : Data,
}
#[derive(Deserialize , Debug)]
#[allow(dead_code)]
pub struct Data{
    #[serde(rename = "Media")]
    pub media : AniDetails,
}
#[derive(Deserialize , Debug)]
#[allow(dead_code)]
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
    #[serde(rename = "startDate")]
    pub start_date : Date,
}
#[derive(Deserialize , Debug)]
#[allow(dead_code)]
pub struct AniTitle{
    pub english : Option<String>,
    pub romaji : Option<String>,
    pub native : Option<String>
}
#[derive(Deserialize , Debug)]
#[allow(dead_code)]
pub struct CoverImage{
    #[serde(rename = "extraLarge")]
    pub extra_large : String,
}
#[derive(Deserialize , Debug)]
#[allow(dead_code)]
pub struct Date{
    pub year : Option<i64>,
    pub month : Option<i64>,
    pub day : Option<i64>,
}
#[derive(Deserialize , Debug)]
pub struct Squerry{
    querry : String,
}

#[derive(Serialize , Debug)]
pub struct Anime{
    pub id : i64,
    pub des : String,
    pub ep : Option<i64>,
    pub title : String,
    pub b_image : Option<String>,
    pub c_image : String,
    pub genres : Vec<String>,
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


const ANI_LIST_API_LINK  : &str = "https://graphql.anilist.co";

// TODO : all this functions
async fn fetch_anilist<T>(payload : &Value) -> Result<T , String> where T: DeserializeOwned{
    let client = Client::new();
    let response = match client
        .post(ANI_LIST_API_LINK)
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

pub async fn ani_search(
    State(pool): State<sqlx::SqlitePool>,
    Json(squerry): Json<Squerry>
) -> (StatusCode , Json<AppResponse<Vec<Anime>>>){

    let search =  search_ani_title(&pool , &squerry.querry).await;


    match search{
        Ok(ele) => {

            if ele.is_empty(){
                println!("Fetch branch");

                let qry = r#"
                    query ($search: String, $status: MediaStatus, $page: Int, $perPage: Int) {
                        Page (page: $page, perPage: $perPage) {
                            media (search: $search, type: ANIME, status: $status) {
                                id
                                title {
                                    romaji
                                    english
                                    native
                                }
                                coverImage {
                                    large
                                    extraLarge
                                }
                                startDate {
                                    year
                                    month
                                    day
                                }
                                bannerImage
                                description
                                episodes
                                status
                                genres
                            }
                        }
                    }
                    "#;

                    let payload = json!({
                        "query" : qry,
                        "variables" : {
                            "search" : squerry.querry,
                            "page" : 1,
                            "perPage" : 10
                        }
                    });
                    let response1 = match fetch_anilist::<serde_json::Value>(&payload).await{
                        Ok(ele) => {
                            println!("{:#?}", ele);
                        }
                        Err(error) =>{
                            return (StatusCode::INTERNAL_SERVER_ERROR , Json(AppResponse::err(&error.to_string())));
                        }
                    };
                    // let response = match fetch_anilist::<Response>(&payload).await{
                    //     Ok(ele) => ele,
                    //     Err(error) => {
                    //         return (StatusCode::INTERNAL_SERVER_ERROR , Json(AppResponse::err(&error.to_string())));
                    //     }
                    // };

                    // fetching from api complete now to 
                    // insert stuff from api to db


                    let something : Vec<AniDetails>;

                    // when we increase the fetch result from
                    // one to many we will first insert stuff in
                    // loop even the learned titles
                    // and then after the insertion is complete then we 
                    // build the Vec<Anime> all at once 

                    let mut inserted : Vec<i64> ;

                    // for one_out_of_this in something {
                    //     match insert_ani_details(one_out_of_this , &pool).await{
                    //         Ok(ele) => inserted.push(ele),
                    //         Err(error) => {
                    //             println!("Failed to insert this {:#?} " , ani_detail.title.english);
                    //         }
                    //     };
                    // }

                    /*********************************************************************/
                    /*       This was when we only took one anime from the api           */
                    /*     //                                                            */
                    /*     // BUILD ANIME WHEN WE HAVE LEARNED A NEW TITLE               */
                    /*     //                                                            */
                    /*     // again we build entirely from our db cache                  */
                    /*     // it is slow as directly building it from response           */
                    /*     // from api but it is straight forward                        */
                    /*     // and requires only one duplication for this                 */
                    /*     // stuff                                                      */
                    /*********************************************************************/

                    //
                    // BUILD AFTER JUST INSERTING A NEW ANIME IN DB CACHE

                    // Here we have jsut inserted stuff in db
                    // and now its time to fetch those exact same
                    // stuff we just inserted back and build the
                    // anime struct . Yes again we have repeat the
                    // build stuff from db but it is simpler..
                    //
                    let mut animes : Vec<Anime> = Vec::with_capacity(ele.len());
                    // for e in inserted.into_iter(){
                    //     if let Ok(some) = fetch_db_by_id(&pool , e).await{
                    //         animes.push(some);
                    //     }
                    // }
                    return (StatusCode::OK , Json(AppResponse::ok("Found", animes)))
            } else{ 
                //
                //  BUILD ENTIRLY FROM DB CACHE
                //
                // this else block is for when we have stuff 
                // in db already and the title matches. We 
                // entirely build from cached db stuff...
                let mut animes : Vec<Anime> = Vec::with_capacity(ele.len());
                //
                // original no oxidised rust 
                //
                for e in ele.into_iter(){
                    if let Ok(some) = fetch_db_by_id(&pool , e).await{
                        animes.push(some);
                    }
                    // i guess we should skip the errors by using a if let thing 
                    // and also it is not fair to discard the successfull animes
                    //
                    //  return (StatusCode::INTERNAL_SERVER_ERROR , Json(AppResponse::err(&error.to_string())));
                }
                return (StatusCode::OK , Json(AppResponse::ok("Found", animes)))
            }
        },
        Err(error) => {
            return (StatusCode::INTERNAL_SERVER_ERROR , Json(AppResponse::err(&error.to_string())))
        }
    }
}

#[allow(dead_code)]
pub async fn get_top_banners(){
    todo!("TODO : get top banners")
}
// TODO : all this functions
#[allow(dead_code)]
pub async fn get_trending() {
    todo!("TODO : get trending")
}

// TODO : all this functions
#[allow(dead_code)]
pub async fn get_all_time() {
    todo!("TODO : get all time")
}

// TODO : all this functions
#[allow(dead_code)]
pub async fn get_current_season() {
    todo!("TODO : get current season")

}

// TODO : all this functions
#[allow(dead_code)]
pub async fn estimated_current_season_schedule(){
    todo!("TODO : get current season schedule")
}

// TODO : all this functions
#[allow(dead_code , unused_variables)]
pub async fn get_all_season_for(Json(squerry) : Json<Squerry>) {
    todo!("TODO : get all season for")
}

