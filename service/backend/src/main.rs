use actix_files::Files;
use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder, error, get, middleware, post, web,
};
use serde::Deserialize;

use r2d2_sqlite::SqliteConnectionManager;

mod db;
use db::{Pool, create_user, get_db_conn, select_password};

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
) -> Result<impl Responder, Error> {
    let form = form.into_inner();
    let conn = get_db_conn(pool).await?;
    let name = form.username.clone();
    let typed_password = form.password.clone();
    println!("{name} {typed_password}");
    let password = web::block(move || select_password(conn, &name))
        .await?
        .map_err(error::ErrorInternalServerError)?;

    if (password == typed_password) {
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
            .service(Files::new("/register", "../frontend").index_file("login.html"))
            .route("/hey", web::get().to(manual_hello))
            .service(newuser)
            .service(check_credentials)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
