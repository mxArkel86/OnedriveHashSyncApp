use actix_files::NamedFile;
use actix_files::Files;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpServer;
use actix_web::dev::Server;
use actix_web::dev::ServerHandle;
use async_std::task::JoinHandle;
use cursive::view::Nameable;
use cursive::view::Resizable;
use cursive::view::Scrollable;
use tokio::runtime::Runtime;
use tokio::task;

#[get("/favicon.ico")]
async fn favicon() -> impl Responder {
    return NamedFile::open("statics/favicon.ico").unwrap();
}

#[get("/auth")]
async fn authentication(req: HttpRequest) -> impl Responder {
    println!("URL={}", req.query_string());
    let code = req.query_string().split_at("code=".len()).1;
    println!("code={}", code);
    let (access_token, refresh_token) = get_access_token(code).await;
    write_option("access_token", &access_token);
    write_option("refresh_token", &refresh_token);
    return "Success";
}

#[get("/")]
async fn index() -> impl Responder {
    
    return NamedFile::open("statics/index.html").unwrap();
}

async fn start_web_server(){
    println!("server is running at 127.0.0.1");

    let server = HttpServer::new(|| {
        App::new()
            .service(index)
            .service(web::redirect("/login", get_authentication_url()))
            .service(favicon)
            .service(authentication)
        //.service(Files::new("/static", "statics/").prefer_utf8(true))
        })
        .bind(("127.0.0.1", 8080))
        .unwrap()
        .run().await.unwrap();
    
    }