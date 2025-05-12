use actix_web::{middleware, get, post, web, error, Error, App, HttpResponse, HttpServer, Responder};
use actix_files::Files;
use serde::Deserialize;

use r2d2_sqlite::SqliteConnectionManager;

mod db;
use db::{Pool, create_user};



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

#[post("/newuser")]
async fn newuser(
    pool: web::Data<Pool>,
    form: web::Form<FormInput>,
) -> Result<impl Responder, Error> {
    let form = form.into_inner();
    let pool = pool.clone();
    
    let conn = web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)?; 

    let name = form.username.clone();
    let password = form.password.clone();
    println!("{name} {password}");
    web::block(move || 
        create_user(conn, &name, &password))
    .await?
    .map_err(error::ErrorInternalServerError);

    Ok(HttpResponse::Ok().body("User created!"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let manager = SqliteConnectionManager::file("parcerotv.db");
    let pool = Pool::new(manager).unwrap();


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(echo)
            .service(Files::new("/login", "../frontend").index_file("login.html"))
            .route("/hey", web::get().to(manual_hello))
            .service(newuser)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}