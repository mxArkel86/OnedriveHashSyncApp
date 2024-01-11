
use json::JsonValue;

use crate::{config, hash_util::OSEntry};

pub async fn get_onedrive_items(access_token: &str, path: &str)->Result<(Vec<JsonValue>, Vec<JsonValue>), (String, String)>{
    let doc = fetch_folder(access_token,path).await?;
    let values = doc["value"].clone();

    let mut folders = Vec::<JsonValue>::new();
    let mut files = Vec::<JsonValue>::new();

    for item in values.members(){
        //let name_node = item["name"].clone();
        if item.has_key("file"){
            files.push(item.clone());
        }else{
            folders.push(item.clone());
        }
    }

    return Ok((folders, files));
}

pub fn merge_onedrive_file_data(data1:JsonValue, data2:JsonValue)->JsonValue{
    let mut doc = data1;
    for obj in data2["value"].members(){
        doc["value"].push(obj.clone()).unwrap();
    }
    return doc;
}

pub async fn fetch_folder(access_token: &str, path: &str) -> Result<JsonValue, (String, String)> {
    let client = reqwest::Client::builder().build().unwrap();

    

    let mut addition = String::new();
    if !path.eq("root"){
        addition = format!(":{}:", String::from(path).replace("root", ""));
    }

    let mut url: String = format!("https://graph.microsoft.com/v1.0/me/drive/root{}/children", addition);

    let mut return_json = JsonValue::new_object();
    let mut loop_inst = 0;
    loop {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", access_token.parse().unwrap());

        let request = client
        .request(
            reqwest::Method::GET,
            url,
        )
        .headers(headers);

        let response = request.send().await.unwrap();
        let body = response.text().await.unwrap();
    
    
        let json_obj = json::parse(&body).unwrap();
        
        if json_obj.has_key("error"){
            
            return Err((json_obj["error"]["code"].to_string(), json_obj["error"]["message"].to_string()));
        }
        
        if loop_inst==0{
            return_json = json_obj.clone();
        }else{
            return_json = merge_onedrive_file_data(return_json, json_obj.clone());
        }
        
        if !json_obj.has_key("@odata.nextLink"){
            break;
        }

        url = json_obj["@odata.nextLink"].as_str().unwrap().to_string();
        loop_inst+=1;
    }
    return Ok(return_json);
}

pub async fn refresh_access_token(refresh_token: &str) -> (String, String) {
    let client_id = config::read_option("client_id");
    let client_secret = config::read_option("client_secret");

    let client = reqwest::Client::builder().build().unwrap();

    let mut params = std::collections::HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("refresh_token", refresh_token);
    params.insert("client_secret", client_secret.as_str());
    params.insert("redirect_uri", "http://localhost:8080/auth");
    params.insert("scope", "Files.Read offline_access");
    params.insert("client_id", client_id.as_str());

    let request = client
        .request(
            reqwest::Method::POST,
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        )
        .form(&params);

    let response = request.send().await.unwrap();
    let body = response.text().await.unwrap();

    let data = json::parse(&body).unwrap();
    
    let access_token = data["access_token"].clone();
    let refresh_token = data["refresh_token"].clone();

    return (
        access_token.to_string(),
        refresh_token.to_string()
    );
}

pub async fn get_access_token(code: &str) -> (String, String) {
    let client_id = config::read_option("client_id");
    let client_secret = config::read_option("client_secret");
    let client = reqwest::Client::builder().build().unwrap();

    let mut params = std::collections::HashMap::new();
    params.insert("grant_type", "authorization_code");
    params.insert("code", code);
    params.insert("client_secret", client_secret.as_str());
    params.insert("redirect_uri", "http://localhost:8080/auth");
    params.insert("scope", "Files.Read offline_access");
    params.insert("client_id", client_id.as_str());

    let request = client
        .request(
            reqwest::Method::POST,
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        )
        .form(&params);

    let response = request.send().await.unwrap();
    let body = response.text().await.unwrap();

    let data = json::parse(&body).unwrap();
    let access_token = data["access_token"].clone();
    let refresh_token = data["refresh_token"].clone();

    return (
        access_token.to_string(),
        refresh_token.to_string()
    );
}

pub fn get_authentication_url() -> String {
    return String::from("https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id=76fa6baf-c6ee-434b-80dd-8e4daf864298&response_type=code&redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fauth&response_mode=query&scope=Files.Read%20offline_access");
}
