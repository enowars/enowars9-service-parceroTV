use std::path::PathBuf;

use actix_files::Files;
use actix_files::NamedFile;
use actix_multipart::form::MultipartForm;
use actix_session::config::PersistentSession;
use actix_web::HttpRequest;
use actix_web::cookie::time::Duration;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header;
use actix_web::middleware;
use actix_web::{App, Error, HttpResponse, HttpServer, Responder, error, get, post, web};
mod forms;
use backend::get_path;
use backend::get_thumbnail_path;
use backend::sanitize_title;
use backend::save_thumbnail;
use backend::save_video;

use db::{
    create_comment, get_all_videos, increase_view_count_db, is_video_private,
    select_comments_by_video_id, select_my_videos, select_private_videos_by_userid, select_user_id,
    select_user_info, select_user_info_with_name, select_video_by_path, select_videos_by_userid,
    update_about_user, update_dislike_db, update_like_db, user_has_permission, select_shorts
};

mod shorts_lib;
use shorts_lib::save_short;

use forms::UpdateAboutForm;
use forms::{CommentForm, FormInput, VideoForm};

use r2d2_sqlite::SqliteConnectionManager;
mod db;
use actix_session::config::CookieContentSecurity;
use actix_session::storage::CookieSessionStore;
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::{Key, SameSite};
use db::{Pool, create_user, get_db_conn, insert_video, select_password};
use rusqlite::types::Null;

use crate::db::get_like_status_db;
use crate::db::insert_short;
use crate::forms::ShortsForm;
use crate::shorts_lib::save_caption;

const SESSION_LIFETIME_MINUTES: i64 = 15;

macro_rules! redirect {
    ($path:expr) => {
        HttpResponse::SeeOther()
            .append_header(("Location", $path))
            .finish()
    };
}

macro_rules! static_page {
    ($route:expr, $file:expr) => {
        actix_files::Files::new($route, "../frontend")
            .index_file($file)
            .use_last_modified(true)
            .prefer_utf8(true)
            .disable_content_disposition()
    };
}

#[get("/")]
async fn start_page(session: Session) -> Result<HttpResponse, Error> {
    if let Ok(Some(_user_id)) = session.get::<u32>("user_id") {
        Ok(redirect!("/app/home"))
    } else {
        Ok(redirect!("/login"))
    }
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
    let maybe_password = web::block(move || select_password(conn, &name))
        .await?
        .map_err(error::ErrorInternalServerError)?;

    let password = match maybe_password {
        Some(password) => password,
        None => return Ok(HttpResponse::Unauthorized().body("User not auth!")),
    };

    if password == typed_password {
        let conn = get_db_conn(&pool).await?;
        let user_id = web::block(move || select_user_id(conn, &name_clone))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        session.insert("user_id", user_id).unwrap();
        return Ok(redirect!("/app/home"));
    } else {
        return Ok(HttpResponse::Unauthorized().body("User not auth!"));
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
    let _ = web::block(move || create_user(conn, &name, &password))
        .await?
        .map_err(error::ErrorInternalServerError);

    Ok(redirect!("/login"))
}

async fn serve_file_or_reject(session: Session, path: &str) -> Result<impl Responder, Error> {
    if let Ok(Some(_user_id)) = session.get::<i32>("user_id") {
        let html_content = std::fs::read_to_string(path)
            .unwrap_or_else(|_| "<h1>404: File Not Found</h1>".to_string());
        Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(html_content))
    } else {
        Ok(redirect!("/"))
    }
}

#[get("/myprofile")]
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

#[get("/users")]
async fn users(session: Session) -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/users.html").await
}

#[get("/playlist")]
async fn playlist(session: Session) -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/playlist.html").await
}

#[get("/shorts")]
async fn shorts(session: Session) -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/shorts.html").await
}

#[get("/no_permission")]
async fn no_permission(session: Session) -> Result<impl Responder, Error> {
    serve_file_or_reject(session, "../frontend/no_permission.html").await
}

#[get("/private/{path:.*}")]
async fn private_videos(
    req: HttpRequest,
    session: Session,
    pool: web::Data<Pool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let path = path.into_inner();
        let private_path = format!("private/{}", path);
        let path_clone = private_path.clone();
        let has_permission =
            web::block(move || user_has_permission(&conn, &user_id, &private_path))
                .await?
                .map_err(ErrorInternalServerError)?;
        if !has_permission {
            return Ok(
                HttpResponse::Unauthorized().body("You have no permission to see this video")
            );
        }
        let file_path = PathBuf::from(format!("../data/{}", path_clone));
        return Ok(NamedFile::open(file_path)?.into_response(&req));
    } else {
        Ok(redirect!("/"))
    }
}

#[post("create_video")]
async fn create_video(
    pool: web::Data<Pool>,
    session: Session,
    MultipartForm(video_form): MultipartForm<VideoForm>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<u32>("user_id") {
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
        Ok(redirect!("/"))
    }
}

#[post("/create_short")]
async fn create_short(
    pool: web::Data<Pool>,
    session: Session,
    MultipartForm(short_form): MultipartForm<ShortsForm>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<u32>("user_id") {
        let short_to_save = short_form.file.file.reopen()?;
        let captions = if short_form.captions.as_str().is_empty() {
            None
        } else {
            Some(short_form.captions.clone())
        };
        let short_path = save_short(short_to_save)?;
        let caption_path = match captions.as_ref() {
                    Some(captions) => Some(save_caption(captions, *short_form.translate_to_spanish, *short_form.duration)?),
                    None => None,
                };

        let conn = get_db_conn(&pool).await?;
        let _ = web::block(move || {
            let captions_ref = captions.as_deref();
            insert_short(
                conn,
                &short_form.name,
                &short_form.description,
                &short_path,
                caption_path.as_deref(),
                captions_ref,
                &user_id,
            )
        })
        .await?
        .map_err(error::ErrorInternalServerError);
        Ok(redirect!("/app/shorts"))
    } else {
        Ok(redirect!("/"))
    }
}

#[post("/post_comment")]
async fn post_comment(
    pool: web::Data<Pool>,
    session: Session,
    form: web::Form<CommentForm>,
    req: HttpRequest,
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

        let referer = req
            .headers()
            .get(header::REFERER)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("/");

        Ok(HttpResponse::SeeOther()
            .append_header((header::LOCATION, referer))
            .finish())
    } else {
        Ok(redirect!("/"))
    }
}

#[get("/get_like_status/{video_id}")]
async fn get_like_status(
    session: Session,
    pool: web::Data<Pool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let video_id = path.into_inner();
        let like_status = web::block(move || get_like_status_db(&conn, &user_id, &video_id))
            .await?
            .map_err(ErrorInternalServerError)?;

        Ok(HttpResponse::Ok().json(like_status))
    } else {
        Ok(redirect!("/"))
    }
}

#[post("/update_like/{video_id}")]
async fn update_like(
    session: Session,
    pool: web::Data<Pool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let video_id = path.into_inner();
        let has_updated = web::block(move || update_like_db(&conn, &user_id, &video_id))
            .await?
            .map_err(ErrorInternalServerError)?;
        if has_updated {
            Ok(HttpResponse::Ok().body("Like updated successfully"))
        } else {
            Ok(HttpResponse::BadRequest().body("Failed to update like"))
        }
    } else {
        Ok(redirect!("/"))
    }
}

#[post("/update_dislike/{video_id}")]
async fn update_dislike(
    session: Session,
    pool: web::Data<Pool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let video_id = path.into_inner();
        let has_updated = web::block(move || update_dislike_db(&conn, &user_id, &video_id))
            .await?
            .map_err(ErrorInternalServerError)?;
        if has_updated {
            Ok(HttpResponse::Ok().body("Dislike updated successfully"))
        } else {
            Ok(HttpResponse::BadRequest().body("Failed to update dislike"))
        }
    } else {
        Ok(redirect!("/"))
    }
}

#[post("/increase_view_count/{video_id}")]
async fn increase_view_count(
    session: Session,
    pool: web::Data<Pool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let video_id = path.into_inner();
        let _ = web::block(move || increase_view_count_db(&conn, &video_id))
            .await?
            .map_err(ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().body("View count increased successfully"))
    } else {
        Ok(HttpResponse::BadRequest().body("Failed to increase view count"))
    }
}

//API
#[get("/api/fetch_all_videos")]
async fn fetch_all_videos(
    pool: web::Data<Pool>,
    session: Session,
) -> Result<impl Responder, Error> {
    if let Ok(Some(_user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;

        let videoss = web::block(move || get_all_videos(conn))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(videoss))
    } else {
        Ok(redirect!("/"))
    }
}

#[get("/get_my_videos")]
async fn get_my_videos(pool: web::Data<Pool>, session: Session) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;

        let videoss = web::block(move || select_my_videos(conn, &user_id))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(videoss))
    } else {
        Ok(redirect!("/"))
    }
}

#[get("/get_videos/{user_id}")]
async fn get_videos_by_userid(
    pool: web::Data<Pool>,
    session: Session,
    user_id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(_user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let user_id = user_id.into_inner();
        let videoss = web::block(move || select_videos_by_userid(conn, user_id))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(videoss))
    } else {
        Ok(redirect!("/"))
    }
}

#[get("/get_private_videos/{user_id}")]
async fn get_private_videos_by_userid(
    pool: web::Data<Pool>,
    session: Session,
    user_id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(_user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let user_id = user_id.into_inner();
        let videoss = web::block(move || select_private_videos_by_userid(conn, user_id))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(videoss))
    } else {
        Ok(redirect!("/"))
    }
}

#[get("/get_comments/{video_id}")]
async fn get_comments(
    pool: web::Data<Pool>,
    session: Session,
    video_id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(_user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let video_id = video_id.into_inner();
        let comments = web::block(move || select_comments_by_video_id(conn, &video_id))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(comments))
    } else {
        Ok(redirect!("/"))
    }
}

#[get("/get_user_info/{id}")]
async fn get_user_info(
    pool: web::Data<Pool>,
    session: Session,
    id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(_user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let id = id.into_inner();
        let user_info = web::block(move || select_user_info(conn, &id))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(user_info))
    } else {
        Ok(redirect!("/"))
    }
}

#[get("/get_user_info_with_name/{name}")]
async fn get_user_info_with_name(
    pool: web::Data<Pool>,
    session: Session,
    name: web::Path<String>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(_user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let name = name.into_inner();
        let user_info = web::block(move || select_user_info_with_name(conn, &name))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(user_info))
    } else {
        Ok(redirect!("/"))
    }
}

#[get("/get_video_info/{path:.*}")]
async fn get_video_info(
    pool: web::Data<Pool>,
    session: Session,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let has_permission = if path.starts_with("videos/") {
            true
        } else {
            user_has_permission(&conn, &user_id, &path).map_err(ErrorInternalServerError)?
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
        Ok(redirect!("/"))
    }
}

#[get("/get_my_profile")]
async fn get_my_profile(session: Session, pool: web::Data<Pool>) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let user_info = web::block(move || select_user_info(conn, &user_id))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(user_info))
    } else {
        Ok(redirect!("/"))
    }
}

#[get("/get_shorts")]
async fn get_shorts(session: Session, pool: web::Data<Pool>) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let conn = get_db_conn(&pool).await?;
        let shorts_info = web::block(move || select_shorts(conn, &user_id))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(shorts_info))
    } else {
        Ok(redirect!("/"))
    }
}


#[get("/shorts/{filename}")]
async fn stream_shorts(req: HttpRequest, path: web::Path<String>, session: Session) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let filename = path.into_inner();

    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Ok(HttpResponse::BadRequest().body("Invalid filename"));
    }

    let mut file_path = PathBuf::from("../data/shorts/");
    file_path.push(&filename);

    let file = NamedFile::open(file_path)?;
    Ok(file.use_last_modified(true).prefer_utf8(true).into_response(&req))
    } else {
        Ok(redirect!("/"))
    }
}

#[post("/update_about")]
async fn update_about(
    session: Session,
    pool: web::Data<Pool>,
    aboutform: web::Form<UpdateAboutForm>,
) -> Result<impl Responder, Error> {
    if let Ok(Some(user_id)) = session.get::<i32>("user_id") {
        let about = aboutform.into_inner().about;
        let conn = get_db_conn(&pool).await?;
        let _ = web::block(move || update_about_user(conn, &about, &user_id))
            .await?
            .map_err(error::ErrorInternalServerError)?;
        Ok(redirect!("app/myprofile"))
    } else {
        Ok(redirect!("/"))
    }
}

fn session_middleware() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
        .cookie_name(String::from("session"))
        .cookie_secure(false)
        .session_lifecycle(
            PersistentSession::default().session_ttl(Duration::minutes(SESSION_LIFETIME_MINUTES)),
        )
        .cookie_same_site(SameSite::Strict)
        .cookie_content_security(CookieContentSecurity::Private)
        .cookie_http_only(true)
        .build()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "../data/parcerotv.db".into());
    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Pool::new(manager).unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(session_middleware())
            .wrap(middleware::DefaultHeaders::new().add(("X-Content-Type-Options", "nosniff")))
            .app_data(web::Data::new(pool.clone()))
            .service(newuser)
            .service(start_page)
            .service(check_credentials)
            .service(private_videos)
            .service(static_page!("/login", "login.html"))
            .service(static_page!("/header", "header.html"))
            .service(static_page!("/footer", "footer.html"))
            .service(static_page!("/register", "register.html"))
            .service(static_page!("/aboutus", "about.html"))
            .service(static_page!("/help", "help.html"))
            .service(static_page!("/developers", "developers.html"))
            .service(static_page!("/terms", "terms.html"))
            .service(static_page!("/privacy", "privacy.html"))
            .service(static_page!("/jobs", "jobs.html"))
            .service(Files::new("/js", "../frontend/js/").show_files_listing())
            .service(Files::new("/css", "../frontend/css/").show_files_listing())
            .service(Files::new("/assets", "../frontend/assets/").show_files_listing())
            .service(Files::new("/videos", "../data/videos/").show_files_listing())
            .service(Files::new("/thumbnails", "../data/thumbnails/").show_files_listing())
            .service(Files::new("/vtt", "../data/vtt/").show_files_listing())
            .service(
                web::scope("/app")
                    .service(profile)
                    .service(home)
                    .service(videos)
                    .service(upload_video_page)
                    .service(create_video)
                    .service(create_short)
                    .service(users)
                    .service(playlist)
                    .service(shorts),
            )
            .service(fetch_all_videos)
            .service(get_shorts)
            .service(stream_shorts)
            .service(no_permission)
            .service(get_video_info)
            .service(logout)
            .service(get_user_info)
            .service(get_user_info_with_name)
            .service(get_my_profile)
            .service(update_about)
            .service(get_videos_by_userid)
            .service(get_private_videos_by_userid)
            .service(post_comment)
            .service(get_comments)
            .service(get_my_videos)
            .service(get_like_status)
            .service(update_like)
            .service(update_dislike)
            .service(increase_view_count)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
