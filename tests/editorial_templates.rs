use serde_json::json;
use std::collections::HashMap;
use tera::{Context, Result, Tera, Value};

fn markdown(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let source = value.as_str().unwrap_or_default();
    Ok(Value::String(markdown::to_html(source)))
}

fn templates() -> Tera {
    let mut tera = Tera::new("src/templates/**/*").expect("templates should parse");
    tera.register_filter("markdown", markdown);
    tera
}

#[test]
fn editorial_pages_render() {
    let tera = templates();
    let post = json!({
        "title": "Results not Words", "content": "## Proof\nShip the work.",
        "author": "Riley Seaburg", "category": null,
        "publish_date": "2024-05-21 17:18:41", "slug": "results-not-words"
    });
    let mut listing = Context::new();
    listing.insert("posts", &vec![post.clone()]);
    let blog = tera.render("pages/blog.html.tera", &listing).unwrap();
    assert!(blog.contains("Build things that") && blog.contains("Results not Words"));
    let mut article = Context::new();
    article.insert("post", &post);
    let page = tera.render("pages/post.html.tera", &article).unwrap();
    assert!(
        page.contains("All field notes") && page.contains(">Proof</h2>"),
        "unexpected post HTML: {page}"
    );
}

#[test]
fn static_article_uses_same_reading_shell() {
    let tera = templates();
    let context = Context::from_serialize(json!({"title":"Agent", "content":"Built."})).unwrap();
    let page = tera
        .render("pages/static_post.html.tera", &context)
        .unwrap();
    assert!(
        page.contains("Agent — Riley Seaburg") && page.contains("Thanks for reading"),
        "unexpected static HTML: {page}"
    );
}
