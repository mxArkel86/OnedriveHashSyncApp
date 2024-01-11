
use std::path::Path;
use std::{fs, thread};
use std::fs::File;
use std::io::{BufWriter, Write};

use std::time::Duration;
use hash_util::{OSEntry, get_xor_hash_multiple_cmd};
use json::JsonValue;
use tokio::time; 
mod config;
mod onedrive_api;
mod onedrive_util;
mod log_util;
use log_util::log;

mod json_util;

use crate::hash_util::get_xor_hash_cmd;
mod hash_util;


fn write_to_log(buf_writer:&mut BufWriter<File>, txt:String){
    writeln!(buf_writer, "{}", txt).unwrap();
    buf_writer.flush().unwrap();
}

fn dir_to_osobj(dir:&str, val:OSEntry)->OSEntry{
    
    let mut paths = dir.split("/").map(|x| String::from(x)).collect::<Vec<String>>();

    let mut current_obj = val;
    for i in 0..paths.len()-1{
        let i_1 = paths.len()-i-2;
        let folder = paths.get(i_1).unwrap();

        let mut temp_obj = OSEntry::new();
        temp_obj.dirname = folder.to_string();
        temp_obj.subdirectories.push(current_obj);
        current_obj = temp_obj;
        
    }
    return current_obj;
}

fn print_osobj(obj:OSEntry){
    let mut current_obj = obj;
    let mut folders = Vec::<String>::new();
    loop{
        folders.push(current_obj.dirname);
        if let Some(newobj) = current_obj.subdirectories.first(){
            current_obj = newobj.clone();
        }else{
            break;
        }
    }

    println!("object = [{}]", folders.join("/"));
}

fn console_input(input: &str) {
    // Get the console output text view
    log(format!("input=[{}]", input).as_str());
    
    let args = input.trim().split(" ").map(|x| x.to_string()).collect::<Vec<String>>();
    if input.starts_with("sync remote"){
            
            let access_token = config::read_option("access_token");

            let mut dir:String = "root".to_string();
            let new_dir = args.iter().skip(2).cloned().collect::<Vec<String>>().join(" ");
            if args.len()>2{
                dir = format!("root/{}", new_dir);
            }

            log(format!("beginning hash retrieval. Path=[{}]", dir).as_str());

            let ret_val = begin_onedrive_fetch_process(access_token.as_str(), dir.as_str());
            if let Ok(hashes) = ret_val{

                let mut doc = JsonValue::new_object();
                if args.len()>2{
                    let local_raw = fs::read_to_string("remote_hashes.json").unwrap();
                    let local_root = json::parse(&local_raw);
                    let local_obj = OSEntry::from(local_root.unwrap().clone());

                    let mut root_obj = OSEntry::named("root");
                    let in_hashes = dir_to_osobj(new_dir.as_str(), hashes);
                    root_obj.subdirectories.push(in_hashes);


                    log("merging data");
                    doc = Into::<JsonValue>::into(merge_hashdata_recurse(local_obj, root_obj));
                }else{
                    doc = Into::<JsonValue>::into(hashes);
                }
                
                let _ = fs::write("remote_hashes.json", doc.pretty(4));
            }else{
                let (errorcode, errormessage) = ret_val.err().unwrap();
                println!("[{}] {}", errorcode, errormessage);
            }
            println!("finished processing remote hashes");
            
        } else if input.starts_with("sync local"){
            log("beginning hash retrieval");

            let mut dir:String = "/Volumes/Removable T7/OneDrive Backup".to_string();
            let new_dir = args.iter().skip(2).cloned().collect::<Vec<String>>().join(" ");
            if args.len()>2{
                dir = format!("/Volumes/Removable T7/OneDrive Backup/{}", new_dir);
            }

            let ret_val = begin_local_hash_process(dir.as_str());
            if let Ok(hashes) = ret_val{
            let mut doc = JsonValue::new_object();

            if args.len()>2{
                let local_raw = fs::read_to_string("local_hashes.json").unwrap();
                let local_root = json::parse(&local_raw);
                let local_obj = OSEntry::from(local_root.unwrap().clone());

                let mut root_obj = OSEntry::named("root");
                let in_hashes = dir_to_osobj(new_dir.as_str(), hashes);
                root_obj.subdirectories.push(in_hashes);


                log("merging data");
                doc = Into::<JsonValue>::into(merge_hashdata_recurse(local_obj, root_obj));
            }else{
                doc = Into::<JsonValue>::into(hashes);
            }

            let _ = fs::write("local_hashes.json", doc.pretty(4));
        }else{
            let (errorcode, errormessage) = ret_val.err().unwrap();
            println!("[{}] {}", errorcode, errormessage);
        }
            println!("finished processing local hashes");
        
        }
        else if input=="token refresh"{
            log("beginning token refresh");
            let refresh_token_in = config::read_option("refresh_token");

            let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

            rt.block_on(async {
            let (access_token, refresh_token) = onedrive_api::refresh_access_token(refresh_token_in.as_str()).await;
            config::write_option("access_token", access_token.as_str());
            config::write_option("refresh_token", refresh_token.as_str());
            });
            println!("finished token refresh");
        }else if input=="reset local"{
            let root_obj = OSEntry::named("root");
            let doc = Into::<JsonValue>::into(root_obj);
            let _ = fs::write("local_hashes.json", doc.pretty(4));
            println!("reset local hashes file");
        }else if input=="reset remote"{
            let root_obj = OSEntry::named("root");
            let doc = Into::<JsonValue>::into(root_obj);
            let _ = fs::write("remote_hashes.json", doc.pretty(4));
            println!("reset remote hashes file");
        }else if input=="compare"{
            let (diff, moved, renamed, added, removed) = get_hash_differences();

            let file = File::create("log.txt").unwrap();

            // Wrap the file in a buffered writer for better performance
            let mut buf_writer = BufWriter::new(file);
            
            println!("{} differences | {} moved | {} renamed | {} added | {} removed", diff.len(), moved.len(), renamed.len(), added.len(), removed.len());

            for file in diff{
                write_to_log(&mut buf_writer, format!("DIFFERENCE | file={} local_hash={}  remote_hash={}", file.0, file.1, file.2));
            }
            for file in moved{
                write_to_log(&mut buf_writer, format!("MOVED | from={} to={} hash={}", file.0, file.1, file.2));
            }
            for file in renamed{
                write_to_log(&mut buf_writer, format!("RENAMED | from={} to={} hash={}", file.0, file.1, file.2));
            }
            for file in added{
                write_to_log(&mut buf_writer, format!("ADDED | file={} hash={}", file.0, file.1));
            }
            for file in removed{
                write_to_log(&mut buf_writer, format!("REMOVED | file={} hash={}", file.0, file.1));
            }
            
            println!("finished compare");
        }else
        {
            let args = input.trim().split(" ").map(|x| String::from(x)).collect::<Vec<String>>();
            if args[0]=="echo"{
                log(format!("repeating statement: {}", input).as_str());
            }else{
                log("command not found");
            }
        }
}

fn main(){

    // let inst1 = time::Instant::now();
    // let x = get_xor_hash_dir_cmd("/Volumes/Removable T7/OneDrive Backup/Archive");
    // let inst2 = time::Instant::now();

    // //println!("{}", x);
    // println!("cmd took [{}]", inst2.duration_since(inst1).as_millis());
    // return;
    loop{

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        // `read_line` returns `Result` of bytes read
        let a = input.chars().into_iter().take_while(|x| !x.eq(&'\n')).collect::<String>();
        console_input(a.as_str());
    }
}

//                          file differences   files found additionally (0=local, 1=remote)
// hashes changed, file moved, file added remotely, file deleted
fn get_hash_differences()->(Vec<(String, String, String)>, Vec<(String,String, String)>, Vec<(String,String, String)>, Vec<(String,String)>, Vec<(String, String)>){
    let local_raw = fs::read_to_string("local_hashes.json").unwrap();
    let local_root = json::parse(&local_raw);

    let remote_raw = fs::read_to_string("remote_hashes.json").unwrap();
    let remote_root = json::parse(&remote_raw);

    let local_obj = OSEntry::from(local_root.unwrap().clone());
    let remote_obj = OSEntry::from(remote_root.unwrap().clone());

    print_osobj(local_obj.clone());
    print_osobj(remote_obj.clone());

    let (diff, additional_local_prev, additional_remote_prev) = check_file_differences_recurse("root", local_obj, remote_obj);

    let mut files_moved = Vec::<(String, String, String)>::new();
    let mut files_removed = Vec::<(String, String)>::new();
    let mut files_renamed = Vec::<(String, String, String)>::new();
    let mut files_added = Vec::<(String, String)>::new();
    for remote in additional_remote_prev.clone(){
        if let Some(local_pair) = additional_local_prev.iter().find(|local|  remote.1==local.1){
            if remote.0.split("/").last().unwrap()==local_pair.0.split("/").last().unwrap(){
                files_moved.push((local_pair.clone().0, remote.0, remote.1));
            }else if remote.0.rsplit_once("/").unwrap().0==local_pair.0.rsplit_once("/").unwrap().0{ //both in the same directory
                files_renamed.push((local_pair.clone().0, remote.0, remote.1));
            }
            
        }else{
            files_added.push(remote);
        }
    }

    for local in additional_local_prev{
        if additional_remote_prev.iter().find(|remote| remote.0.split("/").last().unwrap()==local.0.split("/").last().unwrap() && remote.1==local.1).is_none(){
            files_removed.push(local);
        }
    }

    return (diff, files_moved, files_renamed, files_added, files_removed);
}

fn get_additional_recurse(dir:&str, dirobj:OSEntry)->Vec<(String,String)>{
    let mut additional = Vec::<(String,String)>::new();

    additional.extend(dirobj.filehashes.iter().map(|x| (format!("{}/{}", dir, x.0), x.1.clone())));

    for subdir in dirobj.subdirectories{
        let recurse_data = get_additional_recurse(format!("{}/{}", dir, subdir.dirname).as_str(), subdir);
        additional.extend(recurse_data);
    }

    return additional;
}

fn merge_hashdata_recurse(base:OSEntry, layer:OSEntry)->OSEntry{
    let mut members_out = Vec::<OSEntry>::new();
    let mut file_hashes_out = Vec::<(String, String)>::new();

    let unique_old_members = base.subdirectories.iter().filter(|x| layer.subdirectories.iter().find(|y| x.dirname==y.dirname).is_none());
    let unique_new_members = layer.subdirectories.iter().filter(|x| base.subdirectories.iter().find(|y| x.dirname==y.dirname).is_none());
    
    for old_member in base.subdirectories.clone(){
        if let Some(similar_new_member) = layer.subdirectories.iter().find(|x| x.dirname==old_member.dirname){
            members_out.push(merge_hashdata_recurse(old_member, similar_new_member.clone()));
        }
    }

    members_out.extend(unique_new_members.cloned());
    members_out.extend(unique_old_members.cloned());

    let unique_old_entries = base.filehashes.iter().filter(|x| layer.filehashes.iter().find(|y| x.0==y.0)==None).cloned();
    let unique_new_entries = layer.filehashes.iter().filter(|x| base.filehashes.iter().find(|y| x.0==y.0)==None).cloned();
    for base_pair in base.filehashes.clone(){
        if let Some(layer_pair) = layer.filehashes.iter().find(|x| x.0==base_pair.0){
            file_hashes_out.push((layer_pair.0.clone(), layer_pair.1.clone()));
        }
    }
    file_hashes_out.extend(unique_new_entries.clone());
    file_hashes_out.extend(unique_old_entries.clone());

    return OSEntry{ dirname: base.dirname, subdirectories: members_out, filehashes: file_hashes_out };;
}

fn check_file_differences_recurse(dir:&str, local:OSEntry, remote:OSEntry)->(Vec<(String, String, String)>, Vec<(String,String)>, Vec<(String,String)>){
    //println!("evaluating {}", dir);
    let mut additional_local = local.filehashes.iter()
        .filter(|x| remote.filehashes.iter().find(|y| y.0==x.0)==None).map(|x| (format!("{}/{}", dir, x.0), x.clone().1)).collect::<Vec<(String, String)>>();

    let mut additional_remote = remote.filehashes.iter()
        .filter(|x| local.filehashes.iter().find(|y| y.0==x.0)==None).map(|x| (format!("{}/{}", dir, x.0), x.clone().1)).collect::<Vec<(String, String)>>();

    //files where they are both there but the hashes are different
    let mut differences = Vec::<(String, String, String)>::new();
    for fhash in local.filehashes{
        let fhash_remote = remote.filehashes.iter()
            .find(|y| y.0==fhash.0 && y.1!=fhash.1);
        
        if fhash_remote.is_some(){
            differences.push((format!("{}/{}", dir, fhash.0), fhash.1, fhash_remote.unwrap().1.clone()));
        }
    }


    let mut similar_dirs = Vec::<(OSEntry, OSEntry)>::new();
    for dir_local in local.subdirectories.clone(){
        let dir_remote = remote.subdirectories.iter().find(|y| y.dirname==dir_local.dirname);
        if dir_remote.is_some(){
            similar_dirs.push((dir_local.clone(), (*dir_remote.unwrap()).clone()));
        }else{
            additional_local.extend(get_additional_recurse(format!("{}/{}", dir, dir_local.dirname).as_str(),dir_local));
        }
    }
    for dir_remote in remote.subdirectories{
        let dir_local = local.subdirectories.iter().find(|y| y.dirname==dir_remote.dirname);
        if dir_local.is_none(){
            additional_remote.extend(get_additional_recurse(format!("{}/{}", dir, dir_remote.dirname).as_str(),dir_remote));
        }
    }

    for dir_combination in similar_dirs{
        let recurse_data = check_file_differences_recurse(format!("{}/{}", dir, dir_combination.0.dirname).as_str(),dir_combination.0, dir_combination.1);
        differences.extend(recurse_data.0);
        additional_local.extend(recurse_data.1);
        additional_remote.extend(recurse_data.2);
    }

    return (differences, additional_local, additional_remote);
}

fn begin_onedrive_fetch_process(access_token:&str, dir:&str)->Result<OSEntry, (String, String)>{
    let hashes = get_remote_hashes_recurse(access_token, dir, 99, 0);
    return hashes;
}

fn begin_local_hash_process(dir:&str)->Result<OSEntry, (String, String)>{
    let hashes = get_local_hashes_recurse(dir, 99, 0);
    return hashes;
}

fn list_directory(current_dir:&str)->(Vec<String>, Vec<String>){
    let mut dirs = Vec::<String>::new();
    let mut files = Vec::<String>::new();

    if let Ok(paths) = fs::read_dir(current_dir){

    for entry in paths{
        
        if let Ok(y) = entry{
            let path = String::from(y.path().to_str().unwrap());
            if let Ok(z) = y.metadata(){
                if z.is_file(){
                    files.push(path);
                }else if z.is_dir(){
                    dirs.push(path);
                }
            }
        }
    }
    }
    return (dirs, files);
}

pub fn get_local_hashes_recurse(current_dir:&str, max_levels:u8, current_level:u8)->Result<OSEntry, (String, String)>{
    log(format!("current folder = [{}] level=[{}/{}]", current_dir, current_level, max_levels).as_str());
    let mut subpaths = Vec::<OSEntry>::new();

    if !Path::new(current_dir).exists(){
        return Err((String::from("pathDoesNotExist"), format!("The path [{}] does not exist", current_dir)));
    }

    let (dirs, files) = list_directory(current_dir);
    if current_level<max_levels{
        for i in 0..dirs.len(){
            let newos = get_local_hashes_recurse(dirs.get(i).unwrap().as_str(), max_levels, current_level+1)?;
            subpaths.push(newos);
        }
    }

    //return_hashes = perform_xor_hashes_async(&files, 4);
    let return_hashes:Vec<(String,String)> = get_xor_hash_multiple_cmd(files, 4).iter().map(|x| (x.0.split("/").last().unwrap().to_string(), x.1.clone())).collect();
    // for i in 0..files.len(){
    //     let filepath = files.get(i).unwrap();
    //     return_hashes.push((filepath.split("/").last().unwrap().to_string(), get_xor_hash_cmd(filepath.as_str())));
    //     print!("entry [{}/{}]", i, files.len());
    //     std::io::stdout().flush().unwrap();
    //     thread::sleep(Duration::from_millis(1));
    //     print!("\r");
    // }

    let dir_name = current_dir.to_string().split("/").last().unwrap().to_string();
    return Ok(OSEntry{ dirname: dir_name, subdirectories: subpaths, filehashes: return_hashes });
}

fn get_remote_hashes_recurse(access_token:&str, current_dir:&str, max_levels:u8, current_level:u8)->Result<OSEntry, (String, String)>{
    
    let mut return_hashes = Vec::<(String,String)>::new();
    let mut subpaths = Vec::<OSEntry>::new();

    let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();

    let (directories, filehashes, errorcode, errormessage) = rt.block_on(async {
        // Call the async function and await its completion
        time::sleep(Duration::from_millis(2000)).await;
        let ret1 = onedrive_util::get_file_hashes(access_token, current_dir).await;
        
        if ret1.is_err(){
            let err_val = ret1.err();
            return (Vec::<String>::new(), Vec::<(String, String)>::new(), err_val.clone().unwrap().0, err_val.unwrap().1);
        }else{
            let ok_val = ret1.ok().unwrap();
            return (ok_val.0, ok_val.1, String::new(), String::new());
        }
    });

    if errorcode.len()>0{
        return Err((errorcode, errormessage));
    }

    log(format!("current folder = [{}] level=[{}/{}] folders = [{}] files = [{}]", current_dir, current_level, max_levels, directories.len(), filehashes.len()).as_str());

    return_hashes.extend(filehashes);

    if current_level<max_levels{
        for subdir in directories{
            let newhashes = get_remote_hashes_recurse(access_token, format!("{}/{}", current_dir, subdir).as_str(), max_levels, current_level+1)?;
            subpaths.push(newhashes);
        }
    }

    return Ok(OSEntry{ dirname: current_dir.split("/").last().unwrap().to_string(), subdirectories: subpaths, filehashes:return_hashes });
}



