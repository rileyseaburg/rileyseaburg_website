use crate::models::Post;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDate;
use tera::{Context, Tera};

const TITLE: &str = "A tetherscript Agent That Can Rebuild Itself While It Runs";
const CONTENT: &str = include_str!("../../content/self_rebuilding_agent.md");
const SLUG: &str = "a-tetherscript-agent-that-can-rebuild-itself-while-it-runs";

pub fn listing_post() -> Post {
    let published = NaiveDate::from_ymd_opt(2026, 7, 12).and_then(|date| date.and_hms_opt(0, 0, 0));
    Post {
        id: None,
        author: Some("Riley Seaburg".into()),
        title: Some(TITLE.into()),
        content: Some(
            "A running tetherscript agent edited, checked, reloaded, and used its own new tool."
                .into(),
        ),
        tags: Some(vec!["tetherscript".into(), "AI agents".into()]),
        publish_date: published,
        status: Some("published".into()),
        image_url: Some("/images/icon.png".into()),
        category: Some("Technology".into()),
        created_at: published,
        updated_at: published,
        slug: Some(SLUG.into()),
    }
}

#[get("/posts/a-tetherscript-agent-that-can-rebuild-itself-while-it-runs")]
pub async fn self_rebuilding_agent(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    context.insert("title", TITLE);
    context.insert("content", CONTENT);
    match tmpl.render("pages/static_post.html.tera", &context) {
        Ok(rendered) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(rendered),
        Err(error) => {
            HttpResponse::InternalServerError().body(format!("Unable to render post: {error}"))
        }
    }
}
