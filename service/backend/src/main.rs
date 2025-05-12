use actix_web::{middleware, get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_files::Files;
use serde::Deserialize;

use r2d2_sqlite::SqliteConnectionManager;
use backend::create_user

mod db;
use db::{Pool};



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
    name: String,
    password: String,
}

#[post("/newuser")]
async fn newuser(pool: web::Data<DbPool>,form: web::Form<FormInput>) -> impl Responder {
    let user = web::block(move || {
        // Obtaining a connection from the pool is also a potentially blocking operation.
        // So, it should be called within the `web::block` closure, as well.
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        create_user(&mut conn, &form.name, &form.password);
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let manager = SqliteConnectionManager::file("../db/parcerotv.db");
    let pool = Pool::new(manager).unwrap();


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(echo)
            .service(Files::new("/login", "../frontend").index_file("login.html"))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}