

use std::fs::{File, self};

use std::ops::{Add, AddAssign};
use std::process::{Command, Stdio};
use std::sync::{Mutex, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;




use json::JsonValue;
use min_max::max;
use std::io::{Read, Write};
use sha1::{Sha1, Digest};

// use self::quick_xor_hash::perform_xor_hash_on_file;

// #[path = "quick_xor_hash"]
// mod quick_xor_hash;

pub struct OSEntry{
    pub dirname:String,
    pub subdirectories:Vec<OSEntry>,
    pub filehashes:Vec<(String, String)>
}
impl OSEntry {
    pub(crate) fn new() -> OSEntry {
        return OSEntry::EMPTY;
    }
    pub(crate) fn named(name:&str) -> OSEntry {
        return OSEntry{ dirname: name.to_string(), subdirectories: Vec::<OSEntry>::new(), filehashes: Vec::<(String, String)>::new() };
    }
    pub const EMPTY: OSEntry = OSEntry{ dirname: String::new(), subdirectories: Vec::<OSEntry>::new(), filehashes: Vec::<(String, String)>::new() };
}
impl PartialEq for OSEntry{
    fn eq(&self, other: &Self) -> bool {
        self.dirname == other.dirname && self.subdirectories == other.subdirectories && self.filehashes == other.filehashes
    }
}

impl Clone for OSEntry {
    fn clone(&self) -> Self {
        OSEntry {
            dirname: self.dirname.clone(),
            subdirectories: self.subdirectories.clone(),
            filehashes: self.filehashes.clone(),
        }
    }
}

impl From<JsonValue> for OSEntry{
    fn from(value: JsonValue) -> Self {
        let mut sublist = Vec::<OSEntry>::new();
        for sub in value["subdirectories"].members(){
            sublist.push(OSEntry::from(sub.clone()));
        }

        let mut filehashes = Vec::<(String, String)>::new();
        for fhash in value["filehashes"].entries(){
            filehashes.push((fhash.0.to_string(), fhash.1.as_str().unwrap().to_string()));
        }

        return OSEntry{ dirname: value["dirname"].to_string(), subdirectories: sublist, filehashes: filehashes };
    }
}

impl Into<JsonValue> for OSEntry{
    fn into(self) -> JsonValue {
        let mut obj = JsonValue::new_object();
        obj.insert("dirname", self.dirname).unwrap();
        
        let mut hasharray = JsonValue::new_object();
        for hash in self.filehashes{
            hasharray.insert(hash.0.as_str(), hash.1.as_str()).unwrap();
        }
        obj.insert("filehashes", hasharray).unwrap();
        
        let mut objarray = JsonValue::new_array();
        for obj in self.subdirectories{
            objarray.push(obj).unwrap();
        }
        obj.insert("subdirectories", objarray).unwrap();
        return obj;
    }
}

// pub fn perform_xor_hashes_async(files:&Vec<String>, workers:usize)->Vec<(String, String)>{
//     let chunk_size = files.len()/workers+1;
//     let mut handles = Vec::<std::thread::JoinHandle<()>>::new();
//     let mut result = Mutex::new(Vec::<(String, String)>::new());
    
//     for chunk in files.chunks(chunk_size) {
//         let handle = thread::spawn(move || {
//             for file in chunk{
//                 let hash = get_xor_hash_cmd(file.as_str());
                
//                 let m1 = result.lock();
//                 m1.unwrap().push((file.to_string(), hash));
//             }
//         });
//         handles.push(handle);
//     }

//     for handle in handles{
//         handle.join();
//     }

//     let guard = result.lock().unwrap();
//     return guard.clone();
//}


pub fn get_xor_hash_cmd(file_path:&str)->String{
    let output = Command::new("quickxorhash")
    .args(&[file_path])
    .stdout(Stdio::piped())
    .output().unwrap();

let hash = String::from_utf8(output.stdout)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)).unwrap();

let hash = hash.split_whitespace().next().unwrap_or_default();

return String::from(hash);
}

pub fn get_xor_hash_multiple_cmd(files:Vec::<String>, instances:usize)->Vec<(String, String)>{
    let mut hashfiles = Vec::<(String, String)>::new();
    let mut handles = Vec::<JoinHandle<Vec<(String, String)>>>::new();

    let index = Arc::new(Mutex::new(0));
    
    let files_len = files.len();

    let chunk_size:usize = files.len()/instances+1;
    let mut chunks = files.chunks(chunk_size).map(|x| Vec::from(x));

    let mut thread_index = 0;
    loop{
        if let Some(chunk) = chunks.next(){
            let conn = index.clone();

            let handle = thread::spawn(move || {
                
                let mut hashes = Vec::<(String, String)>::new();
                for str1 in chunk{
                    let out_hash = get_xor_hash_cmd(str1.as_str());

                    let mut protected_i = conn.lock().unwrap();
                    print!("entry [{}/{}] thread-{}", protected_i, files_len, thread_index);
                    std::io::stdout().flush().unwrap();
                    thread::sleep(Duration::from_millis(1));
                    print!("\r");
                    protected_i.add_assign(1);
                    hashes.push((str1.clone(), out_hash));
                }
                return hashes;
            });
            handles.push(handle);
            thread_index+=1;
        }else{
            break;
        }
    }

    for handle in handles{
        let hashes = handle.join().unwrap();
        hashfiles.extend(hashes);
    }
    
    return hashfiles;
}

// pub fn get_xor_hash(file_path:&str)->[u8;20]{
//     let hash = perform_xor_hash_on_file(file_path);
//     return hash;
// }


pub fn get_sha1_hash_cmd(file_path: &str) -> String {
    let output = Command::new("shasum")
        .args(&["-a", "1", file_path])
        .stdout(Stdio::piped())
        .output().unwrap();
    
    let hash = String::from_utf8(output.stdout)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)).unwrap();
    
    let hash = hash.split_whitespace().next().unwrap_or_default();
    
    return String::from(hash);
}

