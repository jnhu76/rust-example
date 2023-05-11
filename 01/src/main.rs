use axum::{
    routing::get,
    Router,
    extract::{Path, Extension},
    http::{StatusCode, HeaderMap, HeaderValue},
};
use std::{
    convert::TryInto,
    num::NonZeroUsize,
    sync::Arc,
    hash::{Hash, Hasher},
    collections::hash_map::DefaultHasher,
};
use tokio::sync::Mutex;
use anyhow::Result;
use bytes::Bytes;
use lru::LruCache;
use tracing::{info, instrument};
use serde::Deserialize;
use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, percent_encode};
use prost::bytes;
use tower::ServiceBuilder;

mod pb;
mod engine;

use pb::*;
use engine::{Engine, Photon};
use image::ImageOutputFormat;

#[derive(Deserialize)]
struct Params {
    spec: String,
    url: String,
}

type Cache = Arc<Mutex<LruCache<u64, Bytes>>>;

#[warn(dead_code)]
async fn hello() -> String {
    format!("hello world.")
}


#[tokio::main]
async fn main() {

    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 初始化 lru
    let cache: Cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1024).unwrap())));

    let app = Router::new()
        .route("/image/:spec/:url", get(generate))
        .route("/", get(hello))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(cache))
        );

    let addr = "0.0.0.0:8888".parse().unwrap();
    print_test_url("https://images.pexels.com/photos/1562477/pexels-photo-1562477.jpeg?auto=compress&cs=tinysrgb&dpr=3&h=750&w=1260");


    tracing::debug!("listening on {}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}


async fn generate(
    Path(Params {spec, url}): Path<Params>,
    Extension(cache): Extension<Cache>
) -> Result<(HeaderMap, Vec<u8>), StatusCode> {
    let url = percent_decode_str(&url).decode_utf8_lossy();
    let spec: ImageSpec = spec
        .as_str()
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let data = retrieve_image(&url, cache)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // todo： 处理图片
    let mut engine: Photon = data
        .try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    engine.apply(&spec.specs);

    let image = engine.generate(ImageOutputFormat::Jpeg(85));

    info!("Finished processing: image size {}", image.len());


    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("image/jpeg"));
    // Ok(format!("url: {}\n spec:{:#?})", url, spec))
    Ok((headers, image))
}


#[instrument(level="info", skip(cache))]
async fn retrieve_image(url: &str, cache: Cache) -> Result<Bytes> {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let key = hasher.finish();

    let g = &mut cache.lock().await;
    let data = match g.get(&key) {
        Some(v) => {
            info!("Match cache {}", key);
            v.to_owned()
        }
        None => {
            info!("Retrieve url");
            let resp = reqwest::get(url).await?;
            let data = resp.bytes().await?;
            g.put(key, data.clone());
            data
        }
    };

    Ok(data)
}


// 辅助
fn print_test_url(url: &str) {
    use std::borrow::Borrow;

    let spec1 = Spec::new_resize(500, 500, resize::SampleFilter::CatmullRom);
    let spec2 = Spec::new_watermark(20, 20);
    let spec3 = Spec::new_filter(filter::Filter::Marine);
    let image_spec = ImageSpec::new(vec![spec1, spec2, spec3]);
    let s: String = image_spec.borrow().into();
    let test_image = percent_encode(url.as_bytes(), NON_ALPHANUMERIC).to_string();
    println!("test url: http://localhost:8888/image/{}/{}", s, test_image);
}