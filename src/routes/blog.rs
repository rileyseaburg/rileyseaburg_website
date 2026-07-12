use crate::routes::self_rebuilding_agent::{listing_post, self_rebuilding_agent};
use actix_identity::Identity;
use actix_web::{get, web, HttpResponse, Responder};
use tera::{Context, Tera};

pub fn configure(config: &mut web::ServiceConfig) {
    config.service(blog).service(self_rebuilding_agent);
}

#[get("/blog")]
async fn blog(user: Option<Identity>, tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    context.insert("posts", &vec![listing_post()]);
    if let Some(user) = user {
        context.insert("username", &user.id().unwrap());
        context.insert("title", "Blog");
        context.insert("route_name", "blog");
        let rendered = tmpl
            .render("layouts/authenticated/blog.html.tera", &context)
            .unwrap();
        HttpResponse::Ok().body(rendered)
    } else {
        context.insert("route_name", "blog");

        let rendered = tmpl.render("pages/blog.html.tera", &context).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}
