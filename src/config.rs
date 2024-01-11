use std::{fs};
use json::{self};

const CONFIG_PATH:&str = "./syncapp.cfg";

pub fn read_option(name:&str)->String{
    let json_raw = fs::read_to_string(CONFIG_PATH).unwrap();
    let doc = json::parse(json_raw.as_str()).unwrap();

    let options = doc["options"][name].clone();
    return options.to_string();
}
pub fn write_option(key:&str, value:&str){
    let json_raw = fs::read_to_string(CONFIG_PATH).unwrap();
    let mut doc = json::parse(json_raw.as_str()).unwrap();

    let mut options = doc["options"].clone();
    if !options.has_key(key){
        let _ = options.insert(key, value);
    }else{
        options.remove(key);
        let _ = options.insert(key, value);
    }
    doc.remove("options");
    let _  = doc.insert("options", options);
    let _ = fs::write(CONFIG_PATH, doc.pretty(4));
}
pub fn remove_option(key:&str){
    let json_raw = fs::read_to_string(CONFIG_PATH).unwrap();
    let mut doc = json::parse(json_raw.as_str()).unwrap();

    let mut options = doc["options"].clone();
    if options.entries().find(|x| x.0.to_string().eq(key)).is_some(){
        options.remove(key);
    }
    doc.remove("options");
    let _  = doc.insert("options", options);
    let _ = fs::write(CONFIG_PATH, doc.pretty(4));
}
