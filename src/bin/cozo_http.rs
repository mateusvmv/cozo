use actix_cors::Cors;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use cozo::{AttrTxItem, Db};
use cozorocks::DbBuilder;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;

type Result<T> = std::result::Result<T, RespError>;

struct RespError {
    err: anyhow::Error,
}

impl Debug for RespError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.err)
    }
}

impl Display for RespError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl actix_web::error::ResponseError for RespError {}

impl From<anyhow::Error> for RespError {
    fn from(err: anyhow::Error) -> RespError {
        RespError { err }
    }
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about=None)]
struct Args {
    /// Path to the directory to store the database
    #[clap(value_parser, default_value_t = String::from("cozo_db"))]
    path: String,

    /// Address to bind the service to
    #[clap(short, long, default_value_t = String::from("127.0.0.1"))]
    bind: String,

    /// Port to use
    #[clap(short, long, default_value_t = 9070)]
    port: u16,

    /// Temporary database, i.e. will be deleted when the program exits
    #[clap(short, long, default_value_t = false, action)]
    temp: bool,
}

struct AppStateWithDb {
    db: Db,
}

#[post("/tx")]
async fn transact(
    body: web::Json<serde_json::Value>,
    data: web::Data<AppStateWithDb>,
) -> Result<impl Responder> {
    dbg!(&body, &data.db);
    Ok(HttpResponse::Ok().body("transact"))
}

#[post("/txa")]
async fn transact_attr(
    body: web::Json<serde_json::Value>,
    data: web::Data<AppStateWithDb>,
) -> Result<impl Responder> {
    let (attrs, comment) = AttrTxItem::parse_request(&body)?;
    let mut tx = data.db.transact_write()?;
    tx.tx_attrs(attrs)?;
    tx.commit_tx(&comment, false)?;
    Ok(HttpResponse::Ok().body("transact-attr success"))
}

#[post("/q")]
async fn query(
    body: web::Json<serde_json::Value>,
    data: web::Data<AppStateWithDb>,
) -> Result<impl Responder> {
    dbg!(&body, &data.db);
    Ok(HttpResponse::Ok().body("query"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    if args.temp && Path::new(&args.path).exists() {
        panic!(
            "cannot open database at '{}' as temporary since it already exists",
            args.path
        );
    }

    let builder = DbBuilder::default()
        .path(&args.path)
        .create_if_missing(true)
        .destroy_on_exit(args.temp);
    let db = Db::build(builder).unwrap();

    let app_state = web::Data::new(AppStateWithDb { db });

    let addr = (&args.bind as &str, args.port);
    eprintln!("Serving database at {}:{}", addr.0, addr.1);

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .service(query)
            .service(transact)
            .service(transact_attr)
    })
    .bind(addr)?
    .run()
    .await
}
