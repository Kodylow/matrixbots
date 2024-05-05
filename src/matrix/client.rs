use std::path::PathBuf;

use anyhow::bail;
use matrix_sdk::{config::SyncSettings, matrix_auth::MatrixSession, AuthSession, Client};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
struct ClientSession {
    homeserver: String,
    db_path: String,
    passphrase: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FullSession {
    client_session: ClientSession,
    user_session: MatrixSession,
    sync_token: Option<String>,
}

async fn restore_session(session_file: &str) -> anyhow::Result<(Client, Option<String>)> {
    let serialized_session = fs::read_to_string(session_file).await?;
    let FullSession {
        client_session,
        user_session,
        sync_token,
    } = serde_json::from_str(&serialized_session)?;

    let client = Client::builder()
        .homeserver_url(&client_session.homeserver)
        .sqlite_store(&client_session.db_path, Some(&client_session.passphrase))
        .build()
        .await?;

    client.restore_session(user_session).await?;

    Ok((client, sync_token))
}

async fn persist_session(
    session_file: &str,
    db_path: String,
    client: &Client,
    sync_token: Option<String>,
    passphrase: String,
) -> anyhow::Result<()> {
    let user_session = match client.session().ok_or(anyhow::anyhow!("no session"))? {
        AuthSession::Matrix(matrix_session) => matrix_session,
        _ => bail!("auth session is not a matrix session"),
    };
    let client_session = ClientSession {
        homeserver: client.homeserver().to_string(),
        db_path: db_path,
        passphrase,
    };

    let full_session = FullSession {
        client_session,
        user_session,
        sync_token,
    };

    let serialized_session = serde_json::to_string(&full_session)?;
    fs::write(session_file, serialized_session).await?;

    Ok(())
}

pub async fn login_and_sync(
    homeserver_url: String,
    username: String,
    password: String,
    data_dir: PathBuf,
    session_file: PathBuf,
) -> anyhow::Result<Client> {
    let matrix_db_path = data_dir.join("matrix");
    let (client, sync_token) = if session_file.exists() {
        restore_session(session_file.to_str().unwrap()).await?
    } else {
        let client = Client::builder()
            .homeserver_url(homeserver_url)
            .sqlite_store(&matrix_db_path, Some(&password))
            .build()
            .await?;
        client
            .matrix_auth()
            .login_username(&username, &password)
            .initial_device_display_name("command bot")
            .await?;

        persist_session(
            session_file.to_str().unwrap(),
            matrix_db_path.to_str().unwrap().to_string(),
            &client,
            None,
            password,
        )
        .await?;
        (client, None)
    };

    let settings = match sync_token {
        Some(token) => SyncSettings::default().token(token),
        None => SyncSettings::default(),
    };
    client.sync(settings).await?;

    Ok(client)
}
