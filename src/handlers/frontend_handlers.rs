use axum::{
    http::{header::CONTENT_TYPE, HeaderMap},
    response::{Html, IntoResponse, Response},
};

pub async fn html_handler() -> Html<&'static str> {
    let html = include_str!("../frontend/index.html");
    return Html(html);
}

pub async fn javascript_handler() -> Response {
    let javascript = include_str!("../frontend/index.js");
    let mut header = HeaderMap::new();
    header.insert(CONTENT_TYPE, "text/javascript".parse().unwrap());
    return (header, javascript).into_response();
}
