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

#[get("/video/{ok}")]
async fn get_video(path: web::Path<String>) -> Result<NamedFile, Error> {
    let filename = path.into_inner();
    Ok(NamedFile::open("ok.mp4")?)
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
        return Ok(HttpResponse::Ok().body("User authenticated!"));
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

#[get("/app/{filename}")]
async fn protected_route(session: Session, path: web::Path<String>) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let filename = path.into_inner();
        let html_content = std::fs::read_to_string("../frontend/profile.html")
        .unwrap_or_else(|_| "<h1>404: File Not Found</h1>".to_string());
        Ok(actix_web::HttpResponse::Ok().content_type("text/html").body(html_content))
    } else {
        Ok(HttpResponse::Unauthorized().body("Please log in"))
    }
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
    let manager = SqliteConnectionManager::file("parcerotv.db");
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
            .service(protected_route)
            .service(get_video)
            .service(Files::new("/login", "../frontend").index_file("login.html"))
            .service(Files::new("/register", "../frontend").index_file("register.html"))
            .service(Files::new("/js", "../frontend/js").show_files_listing())
            .service(Files::new("/css", "../frontend/css"))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
