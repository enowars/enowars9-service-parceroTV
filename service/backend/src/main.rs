use actix_files::Files;
use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder, error, get, middleware, post, web,
};
use actix_files::NamedFile;
use serde::Deserialize;

use r2d2_sqlite::SqliteConnectionManager;

mod db;
use db::{Pool, create_user, get_db_conn, select_password};
use actix_session::{ SessionMiddleware, Session };
use actix_session::config::{ BrowserSession, CookieContentSecurity };
use actix_session::storage::{ CookieSessionStore };
use actix_web::cookie::{ Key, SameSite };

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}


#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[derive(Deserialize)]
struct FormInput {
    username: String,
    password: String,
}

#[post("/checkcredentials")]
async fn check_credentials(
    pool: web::Data<Pool>,
    form: web::Form<FormInput>,
    session: Session
) -> Result<impl Responder, Error> {
    let form = form.into_inner();
    let conn = get_db_conn(pool).await?;
    let name = form.username.clone();
    let typed_password = form.password.clone();
    println!("{name} {typed_password}");
    let password = web::block(move || select_password(conn, &name))
        .await?
        .map_err(error::ErrorInternalServerError)?;

    println!("{password} und {typed_password}");
    if (password == typed_password) {
        session.insert("user_id", 1).unwrap();
        return Ok(HttpResponse::SeeOther()
        .append_header(("Location", "/app/home"))
        .finish());
    } else {
        return Ok(HttpResponse::Ok().body("User not auth!"));
    }
}

#[post("/newuser")]
async fn newuser(
    pool: web::Data<Pool>,
    form: web::Form<FormInput>,
) -> Result<impl Responder, Error> {
    let form = form.into_inner();
    let conn = get_db_conn(pool).await?;
    let name = form.username.clone();
    let password = form.password.clone();
    println!("{name} {password}");
    web::block(move || create_user(conn, &name, &password))
        .await?
        .map_err(error::ErrorInternalServerError);

    Ok(HttpResponse::Ok().body("User created!"))
}

async fn serve_file_or_reject(session: Session, path: &str) -> Result<impl Responder, Error> {
    if let Ok(Some(_user_id)) = session.get::<i32>("user_id") {
        let html_content = std::fs::read_to_string(path)
            .unwrap_or_else(|_| "<h1>404: File Not Found</h1>".to_string());
        Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(html_content))
    } else {
        Ok(HttpResponse::Unauthorized().body("Please log in"))
    }
}

#[get("/profile")]
async fn profile(session: Session) -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/profile.html").await
}

#[get("/home")]
async fn home(session: Session)  -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/home.html").await
}


#[get("/videos")]
async fn videos(session: Session)  -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/videos.html").await
}

fn session_middleware() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(
        CookieSessionStore::default(), Key::from(&[0; 64])
    )
	.cookie_name(String::from("session")) // arbitrary name
	.cookie_secure(false) // https only
	.session_lifecycle(BrowserSession::default()) // expire at end of session
	.cookie_same_site(SameSite::Strict) 
	.cookie_content_security(CookieContentSecurity::Private) // encrypt
	.cookie_http_only(true) // disallow scripts from reading
	.build()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let manager = SqliteConnectionManager::file("../data/parcerotv.db");
    let pool = Pool::new(manager).unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(session_middleware())
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            .service(newuser)
            .service(check_credentials)
            .service(Files::new("/login", "../frontend").index_file("login.html"))
            .service(Files::new("/register", "../frontend").index_file("register.html"))
            .service(Files::new("/js", "../frontend/js/").show_files_listing())
            .service(Files::new("/css", "../frontend/css/"))
            .service(Files::new("/videos", "../data/videos/").show_files_listing())
            .service(web::scope("/app")
                        .service(profile)
                        .service(home)
                        .service(videos))       
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
