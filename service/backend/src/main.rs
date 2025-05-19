use std::io::Write;
use std::path;
use std::path::PathBuf;

use actix_files::Files;
use actix_files::NamedFile;
use actix_multipart::form::MultipartForm;
use actix_web::error::ErrorInternalServerError;
use actix_web::HttpRequest;
use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder, error, get, middleware, post, web,
};
mod forms;
use backend::get_path;
use backend::get_thumbnail_path;
use backend::save_thumbnail;
use backend::save_video;
use db::create_comment;
use db::get_all_videos;
use db::is_video_private;
use db::select_user_id;
use db::select_video_by_path;
use db::user_has_permission;
use forms::{CommentForm, FormInput, VideoForm};

use r2d2_sqlite::SqliteConnectionManager;
use tempfile::NamedTempFile;
mod db;
use actix_session::config::{BrowserSession, CookieContentSecurity};
use actix_session::storage::CookieSessionStore;
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::{Key, SameSite};
use db::{Pool, create_user, get_db_conn, insert_video, select_password};

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
    let conn = get_db_conn(&pool).await?;
    let name = form.username.clone();
    let name_clone = name.clone();
    let typed_password = form.password.clone();
    println!("{name} {typed_password}");
    let maybe_password = web::block(move || select_password(conn, &name))
        .await?
        .map_err(error::ErrorInternalServerError)?;

    let password = match maybe_password {
        Some(password) => password,
        None => return Ok(HttpResponse::Ok().body("User not auth!")),
    };

    if (password == typed_password) {
        let conn = get_db_conn(&pool).await?;
        let user_id = web::block(move || select_user_id(conn, &name_clone))
        .await?
        .map_err(error::ErrorInternalServerError)?;
        session.insert("user_id", user_id).unwrap();
        return Ok(redirect!("/app/home"));
    } else {
        return Ok(HttpResponse::Ok().body("User not auth!"));
    }
}

#[get("/logout")]
async fn logout(session: Session) -> Result<impl Responder, Error> {
    session.purge();
    Ok(redirect!("/login"))
}

#[post("/newuser")]
async fn newuser(
    pool: web::Data<Pool>,
    form: web::Form<FormInput>,
) -> Result<impl Responder, Error> {
    let form = form.into_inner();
    let conn = get_db_conn(&pool).await?;
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

#[get("/upload")]
async fn upload_video_page(session: Session) -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/upload.html").await
}

#[get("/no_permission")]
async fn no_permission(session: Session) -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/no_permission.html").await
}

#[get("/private/{path:.*}")]
async fn private_videos(req: HttpRequest,
    session: Session,
    pool: web::Data<Pool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let path = path.into_inner();
        let private_path = format!("private/{}", path);
        let path_clone = private_path.clone();
        let has_permission = web::block(move || user_has_permission(&conn, &user_id, &private_path))
            .await?
            .map_err(ErrorInternalServerError)?;

        println!("Has permission {} for user {} with path {}", &has_permission, &user_id, &path_clone);
        if !has_permission {
            return Ok(HttpResponse::Unauthorized().body("You have no permission to see this video"));
        }
        let file_path = PathBuf::from(format!("../data/{}", path_clone));
        return Ok(NamedFile::open(file_path)?.into_response(&req));
    } else {
        Ok(HttpResponse::Unauthorized().body("Please log in"))
    }
}

#[post("create_video")]
async fn create_video(
    pool: web::Data<Pool>,
    session: Session,
    MultipartForm(video_form): MultipartForm<VideoForm>,
) -> Result<impl Responder, Error> {
    println!("We are here");
    if let Ok(Some(user_id)) = session.get::<u32>("user_id") {
        println! {"Video Form, Title {}, Desc.: {}, ",*video_form.name, *video_form.description};
        let file_to_save = video_form.file.file.reopen()?;
        let thumbnail_to_save = video_form.thumbnail.file.reopen()?;
        let path = get_path(
            *video_form.is_private,
            &video_form.name,
            &video_form.file.file,
        );
        let thumbnail_path = get_thumbnail_path(&video_form.name, &video_form.file.file);
        let thumbnail_path_clone = thumbnail_path.clone();
        let path_clone = path.clone();

        let conn = get_db_conn(&pool).await?;

        let _ = web::block(move || {
            insert_video(
                conn,
                &video_form.name,
                &video_form.description,
                &path,
                &thumbnail_path,
                &user_id,
                &video_form.is_private,
                &video_form.location,
            )
        })
        .await?
        .map_err(error::ErrorInternalServerError);

        save_thumbnail(&thumbnail_path_clone, thumbnail_to_save)?;
        save_video(&path_clone, file_to_save)?;
        Ok(redirect!("/app/home"))
    } else {
        Ok(HttpResponse::Unauthorized().body("Please log in"))
    }
}

#[post("/post_comment")]
async fn post_comment(
    pool: web::Data<Pool>,
    session: Session,
    form: web::Form<CommentForm>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let comment_form = form.into_inner();
        let is_private =
            is_video_private(&conn, &comment_form.video_id).map_err(ErrorInternalServerError)?;
        if is_private {
            return Ok(redirect!("/no_permission"));
        }
        let _ = web::block(move || {
            create_comment(
                conn,
                &comment_form.comment,
                &user_id,
                &comment_form.video_id,
            )
        })
        .await?
        .map_err(error::ErrorInternalServerError);
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::Unauthorized().body("Please log in"))
    }
}

//API
#[get("/api/fetch_all_videos")]
async fn fetch_all_videos(
    pool: web::Data<Pool>,
    session: Session,
) -> Result<impl Responder, Error> {
    println!("/api/fetch_all_videos");
    if let Ok(Some(_user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;

        let videoss = web::block(move || get_all_videos(conn))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(videoss))
    } else {
        Ok(HttpResponse::Unauthorized().body("Please log in"))
    }
}

#[get("/get_video_info/{path:.*}")]
async fn get_video_info(
    pool: web::Data<Pool>,
    session: Session,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    println!("/get_video_info/{}", &path);
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let has_permission = if path.starts_with("videos/") {
            true
        } else {
            user_has_permission(&conn, &user_id, &path)
                .map_err(ErrorInternalServerError)?
        };
        if !has_permission {
            return Ok(redirect!("/no_permission"));
        } else {
            let video_info = web::block(move || select_video_by_path(conn, &path))
                .await?
                .map_err(error::ErrorInternalServerError)?;
            Ok(HttpResponse::Ok().json(video_info))
        }
    } else {
        Ok(HttpResponse::Unauthorized().body("Please log in"))
    }
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
            .service(private_videos)
            .service(Files::new("/login", "../frontend").index_file("login.html"))
            .service(Files::new("/register", "../frontend").index_file("register.html"))
            .service(Files::new("/js", "../frontend/js/").show_files_listing())
            .service(Files::new("/css", "../frontend/css/"))
            .service(Files::new("/videos", "../data/videos/").show_files_listing())
            .service(Files::new("/thumbnails", "../data/thumbnails/").show_files_listing())
            .service(
                web::scope("/app")
                    .service(profile)
                    .service(home)
                    .service(videos)
                    .service(upload_video_page)
                    .service(create_video),
            )
            .service(fetch_all_videos)
            .service(no_permission)
            .service(get_video_info)
            .service(logout)
            
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
