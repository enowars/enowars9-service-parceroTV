use std::io::Write;

use actix_files::Files;
use actix_files::NamedFile;
use actix_multipart::form::MultipartForm;
use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder, error, get, middleware, post, web,
};
mod forms;
use backend::get_path;
use backend::save_video;
use db::get_all_videos;
use forms::{FormInput, VideoForm};

use r2d2_sqlite::SqliteConnectionManager;
use tempfile::NamedTempFile;
mod db;
use actix_session::config::{BrowserSession, CookieContentSecurity};
use actix_session::storage::CookieSessionStore;
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::{Key, SameSite};
use db::{Pool, create_user, insert_video, get_db_conn, select_password};


macro_rules! redirect {
    ($path:expr) => {
        HttpResponse::SeeOther()
            .append_header(("Location", $path))
            .finish();
    };
}


#[get("/")]
async fn hello(session: Session) -> Result<HttpResponse, Error> {
    if let Ok(Some(_user_id)) = session.get::<u32>("user_id") {
        Ok(redirect!("/app/home"))
    } else {
        Ok(redirect!("/login"))
    }
}


#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}


#[post("/checkcredentials")]
async fn check_credentials(
    pool: web::Data<Pool>,
    form: web::Form<FormInput>,
    session: Session,
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
        return Ok(redirect!("/app/home"));
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
    let _ = web::block(move || create_user(conn, &name, &password))
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
async fn home(session: Session) -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/home.html").await
}

#[get("/videos")]
async fn videos(session: Session) -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/videos.html").await
}

#[post("create_video")]
async fn create_video(pool:web::Data<Pool>,session: Session, MultipartForm(video_form): MultipartForm<VideoForm>) -> Result<impl Responder, Error> {
    println!("We are here");
    if let Ok(Some(user_id)) = session.get::<u32>("user_id") {
        println!{"Video Form, Title {}, Desc.: {}, ",*video_form.name, *video_form.description};
        let file_to_save = video_form.file.file.reopen()?;
        let path = get_path(*video_form.is_private, &video_form.name, video_form.file.file);
        let path_clone = path.clone();
        
        let conn = get_db_conn(pool).await?;

        let _ = web::block(move || insert_video(conn, &video_form.name, &video_form.description, &path, &user_id, &video_form.is_private))
        .await?
        .map_err(error::ErrorInternalServerError);

        save_video(&path_clone, file_to_save)?;
        Ok(HttpResponse::Ok().body("Video created!"))
    } else {
        Ok(HttpResponse::Unauthorized().body("Please log in"))
    }
}

//API
#[get("/api/fetch_all_videos")]
async fn fetch_all_videos(pool:web::Data<Pool>,session: Session,) -> Result<impl Responder, Error> {
    let conn = get_db_conn(pool).await?;

    let videoss = web::block(move || get_all_videos(conn)).await?.map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(videoss))
}

fn session_middleware() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
        .cookie_name(String::from("session")) 
        .cookie_secure(false)
        .session_lifecycle(BrowserSession::default()) 
        .cookie_same_site(SameSite::Strict)
        .cookie_content_security(CookieContentSecurity::Private) 
        .cookie_http_only(true) 
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
            .service(
                web::scope("/app")
                    .service(profile)
                    .service(home)
                    .service(videos)
                    .service(create_video),
            )
            .service(fetch_all_videos)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
