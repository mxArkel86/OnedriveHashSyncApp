

#[path="onedrive_api.rs"] mod onedrive_api;
use json::JsonValue;
use onedrive_api::get_onedrive_items;



pub async fn get_file_hashes(access_token:&str, dir:&str)-> Result<(Vec<String>, Vec<(String,String)>), (String, String)>{
    let mut subdirectories = Vec::<String>::new();
    let mut hashes = Vec::<(String, String)>::new();

    let (directories, files):(Vec<JsonValue>, Vec<JsonValue>) = get_onedrive_items(access_token, dir).await?;

    for dir in directories{
        subdirectories.push(dir["name"].to_string());
    }

    for file in files{
        hashes.push((file["name"].to_string(), file["file"]["hashes"]["quickXorHash"].to_string()));
    }

    return Ok((subdirectories, hashes));
}