use sentry::integrations::tower::{NewSentryLayer, SentryHttpLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use crate::config::Config;

mod domain {
    use std::fmt::Debug;
    use tokio::fs::File;

    pub enum Branch {
        Master,
        Develop,
    }

    pub const BUILD_ARTIFACT_PATH: &str = "/SeichiAssist/target/SeichiAssist.jar";

    pub const STABLE_BUILD_DIR_PATH: &str = "/builds/stable";
    pub const STABLE_BUILD_FILE_PATH: &str = "/builds/stable/SeichiAssist.jar";

    pub const DEVELOP_BUILD_DIR_PATH: &str = "/builds/develop";
    pub const DEVELOP_BUILD_FILE_PATH: &str = "/builds/develop/SeichiAssist.jar";

    pub trait BuildHandler: Debug + Sync + Send + 'static {
        async fn run_stable_build(&self) -> anyhow::Result<()>;
        async fn run_develop_build(&self) -> anyhow::Result<()>;
        async fn get_stable_build(&self) -> anyhow::Result<File>;
        async fn get_develop_build(&self) -> anyhow::Result<File>;
    }

    #[derive(Debug, Clone)]
    pub struct BuildRepository {}
}

mod infra_repository_impls {
    use crate::domain::Branch;
    use crate::domain::{
        BuildHandler, BuildRepository, BUILD_ARTIFACT_PATH, DEVELOP_BUILD_DIR_PATH,
        DEVELOP_BUILD_FILE_PATH, STABLE_BUILD_DIR_PATH, STABLE_BUILD_FILE_PATH,
    };
    use anyhow::anyhow;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    async fn switch_branch(branch: Branch) -> anyhow::Result<()> {
        let branch_name = match branch {
            Branch::Master => "master",
            Branch::Develop => "develop",
        };

        Command::new("git")
            .args(vec!["switch", branch_name])
            .status()?;
        Command::new("git")
            .args(vec!["pull", "origin", branch_name])
            .status()?;

        Ok(())
    }

    impl BuildHandler for BuildRepository {
        async fn run_stable_build(&self) -> anyhow::Result<()> {
            switch_branch(Branch::Master).await?;
            Command::new("sbt").arg("build").output()?;

            if !Path::new(STABLE_BUILD_DIR_PATH).is_dir() {
                fs::create_dir(STABLE_BUILD_DIR_PATH)?;
            }

            fs::remove_file(STABLE_BUILD_FILE_PATH)?;
            fs::copy(BUILD_ARTIFACT_PATH, STABLE_BUILD_FILE_PATH)?;

            Ok(())
        }

        async fn run_develop_build(&self) -> anyhow::Result<()> {
            switch_branch(Branch::Develop).await?;
            Command::new("sbt").arg("build").status()?;

            if !Path::new(DEVELOP_BUILD_DIR_PATH).is_dir() {
                fs::create_dir(DEVELOP_BUILD_DIR_PATH)?;
            }

            fs::remove_file(DEVELOP_BUILD_FILE_PATH)?;
            fs::copy(BUILD_ARTIFACT_PATH, DEVELOP_BUILD_FILE_PATH)?;

            Ok(())
        }

        async fn get_stable_build(&self) -> anyhow::Result<tokio::fs::File> {
            if Path::new(STABLE_BUILD_FILE_PATH).exists() {
                Ok(tokio::fs::File::open(STABLE_BUILD_FILE_PATH).await?)
            } else {
                Err(anyhow!("SeichiAssist was not built yet."))
            }
        }

        async fn get_develop_build(&self) -> anyhow::Result<tokio::fs::File> {
            if Path::new(DEVELOP_BUILD_FILE_PATH).exists() {
                Ok(tokio::fs::File::open(DEVELOP_BUILD_FILE_PATH).await?)
            } else {
                Err(anyhow!("SeichiAssist was not built yet."))
            }
        }
    }
}

mod presentation {
    use crate::domain::{BuildHandler, BuildRepository};
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::{ErrorResponse, IntoResponse, Response, Result};
    use tokio_util::io::ReaderStream;

    #[tracing::instrument]
    pub async fn get_stable_build_handler(
        State(repository): State<BuildRepository>,
    ) -> Result<impl IntoResponse> {
        match repository.get_stable_build().await {
            Ok(stable_build) => Ok(Response::builder()
                .status(StatusCode::OK)
                .header(
                    "Content-Disposition",
                    "attachment; filename=SeichiAssist.jar",
                )
                .header("Content-Type", "application/java-archive")
                .body(axum::body::Body::from_stream(ReaderStream::new(
                    stable_build,
                )))
                .unwrap()),
            Err(_) => Err(ErrorResponse::from(
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "SeichiAssist was not built yet.",
                )
                    .into_response(),
            )),
        }
    }

    #[tracing::instrument]
    pub async fn get_develop_build_handler(
        State(repository): State<BuildRepository>,
    ) -> Result<impl IntoResponse> {
        match repository.get_develop_build().await {
            Ok(develop_build) => Ok(Response::builder()
                .status(StatusCode::OK)
                .header(
                    "Content-Disposition",
                    "attachment; filename=SeichiAssist.jar",
                )
                .header("Content-Type", "application/java-archive")
                .body(axum::body::Body::from_stream(ReaderStream::new(
                    develop_build,
                )))
                .unwrap()),
            Err(_) => Err(ErrorResponse::from(
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "SeichiAssist was not built yet.",
                )
                    .into_response(),
            )),
        }
    }

    #[tracing::instrument]
    pub async fn publish_stable_build_handler(
        State(repository): State<BuildRepository>,
    ) -> Result<impl IntoResponse> {
        match repository.run_stable_build().await {
            Ok(_) => Ok(StatusCode::OK.into_response()),
            Err(err) => {
                tracing::error!("{:}", err);
                Err(ErrorResponse::from(
                    StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                ))
            }
        }
    }

    #[tracing::instrument]
    pub async fn publish_develop_build_handler(
        State(repository): State<BuildRepository>,
    ) -> Result<impl IntoResponse> {
        match repository.run_develop_build().await {
            Ok(_) => Ok(StatusCode::OK.into_response()),
            Err(err) => {
                tracing::error!("{:}", err);
                Err(ErrorResponse::from(
                    StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                ))
            }
        }
    }
}

mod config {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Config {
        pub http_port: u16
    }

    impl Config {
        pub async fn from_environment() -> envy::Result<Config> {
            envy::from_env::<Config>()
        }
    }
}

#[tokio::main]
async fn main() {
    use crate::domain::BuildHandler;
    use crate::domain::BuildRepository;
    use crate::presentation::{
        get_develop_build_handler, get_stable_build_handler, publish_develop_build_handler,
        publish_stable_build_handler,
    };
    use axum::routing::{get, post};
    use axum::Router;
    use tokio::net::TcpListener;

    tracing_subscriber::registry()
        .with(sentry::integrations::tracing::layer())
        .with(
            tracing_subscriber::fmt::layer().with_filter(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            )),
        )
        .init();

    let config = Config::from_environment()
        .await
        .expect("Failed to load config from environment variables.");

    // TODO: Sentryの設定をする
    // let _guard = sentry::init((
    //     "",
    //     sentry::ClientOptions {
    //         release: sentry::release_name!(),
    //         traces_sample_rate: 1.0,
    //         ..Default::default()
    //     },
    // ));

    // sentry::configure_scope(|scope| scope.set_level(Some(sentry::Level::Warning)));

    let layer = tower::ServiceBuilder::new()
        .layer(NewSentryLayer::new_from_top())
        .layer(SentryHttpLayer::with_transaction());

    let build_repository = BuildRepository {};

    build_repository.run_stable_build().await.unwrap();
    build_repository.run_develop_build().await.unwrap();

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.http_port));
    let listener = TcpListener::bind(&addr).await.unwrap();

    let router = Router::new()
        .route("/stable", get(get_stable_build_handler))
        .route("/develop", get(get_develop_build_handler))
        .route("/publish/stable", post(publish_stable_build_handler))
        .route("/publish/develop", post(publish_develop_build_handler))
        .with_state(build_repository)
        .layer(layer);

    axum::serve(listener, router).await.unwrap();
}
