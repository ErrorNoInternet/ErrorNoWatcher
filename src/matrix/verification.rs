use std::time::Duration;

use anyhow::{Context, Result};
use futures::StreamExt;
use log::{error, info, warn};
use matrix_sdk::{
    Client,
    crypto::{Emoji, SasState, format_emojis},
    encryption::verification::{
        SasVerification, Verification, VerificationRequest, VerificationRequestState,
    },
    ruma::{
        UserId,
        events::{
            key::verification::request::ToDeviceKeyVerificationRequestEvent,
            room::message::{MessageType, OriginalSyncRoomMessageEvent},
        },
    },
};
use tokio::time::sleep;

async fn confirm_emojis(sas: SasVerification, emoji: [Emoji; 7]) {
    info!("\n{}", format_emojis(emoji));
    warn!("automatically confirming emojis in 10 seconds");
    sleep(Duration::from_secs(10)).await;
    if let Err(error) = sas.confirm().await {
        error!("failed to confirm emojis: {error:?}");
    }
}

async fn print_devices(user_id: &UserId, client: &Client) -> Result<()> {
    info!("devices of user {user_id}");

    let own_id = client.device_id().context("missing own device id")?;
    for device in client
        .encryption()
        .get_user_devices(user_id)
        .await?
        .devices()
        .filter(|device| device.device_id() != own_id)
    {
        info!(
            "\t{:<10} {:<30} {:<}",
            device.device_id(),
            device.display_name().unwrap_or("-"),
            if device.is_verified() { "✅" } else { "❌" }
        );
    }

    Ok(())
}

async fn sas_verification_handler(client: Client, sas: SasVerification) -> Result<()> {
    info!(
        "starting verification with {} {}",
        &sas.other_device().user_id(),
        &sas.other_device().device_id()
    );
    print_devices(sas.other_device().user_id(), &client).await?;
    sas.accept().await?;

    while let Some(state) = sas.changes().next().await {
        match state {
            SasState::KeysExchanged {
                emojis,
                decimals: _,
            } => {
                tokio::spawn(confirm_emojis(
                    sas.clone(),
                    emojis.context("only emoji verification supported")?.emojis,
                ));
            }
            SasState::Done { .. } => {
                let device = sas.other_device();
                info!(
                    "successfully verified device {} {} with trust {:?}",
                    device.user_id(),
                    device.device_id(),
                    device.local_trust_state()
                );
                print_devices(sas.other_device().user_id(), &client).await?;
                break;
            }
            SasState::Cancelled(info) => {
                warn!("verification cancelled: {}", info.reason());
                break;
            }
            SasState::Created { .. }
            | SasState::Started { .. }
            | SasState::Accepted { .. }
            | SasState::Confirmed => (),
        }
    }

    Ok(())
}

async fn request_verification_handler(client: Client, request: VerificationRequest) {
    info!(
        "accepting verification request from {}",
        request.other_user_id()
    );
    if let Err(error) = request.accept().await {
        error!("failed to accept verification request: {error:?}");
        return;
    }

    while let Some(state) = request.changes().next().await {
        match state {
            VerificationRequestState::Created { .. }
            | VerificationRequestState::Requested { .. }
            | VerificationRequestState::Ready { .. } => (),
            VerificationRequestState::Transitioned { verification } => {
                if let Verification::SasV1(sas) = verification {
                    tokio::spawn(async move {
                        if let Err(error) = sas_verification_handler(client, sas).await {
                            error!("failed to handle sas verification request: {error:?}");
                        }
                    });
                    break;
                }
            }
            VerificationRequestState::Done | VerificationRequestState::Cancelled(_) => break,
        }
    }
}

pub async fn on_device_key_verification_request(
    event: ToDeviceKeyVerificationRequestEvent,
    client: Client,
) -> Result<()> {
    let request = client
        .encryption()
        .get_verification_request(&event.sender, &event.content.transaction_id)
        .await
        .context("request object wasn't created")?;
    tokio::spawn(request_verification_handler(client, request));

    Ok(())
}

pub async fn on_room_message_verification_request(
    event: OriginalSyncRoomMessageEvent,
    client: Client,
) -> Result<()> {
    if let MessageType::VerificationRequest(_) = &event.content.msgtype {
        let request = client
            .encryption()
            .get_verification_request(&event.sender, &event.event_id)
            .await
            .context("request object wasn't created")?;
        tokio::spawn(request_verification_handler(client, request));
    }

    Ok(())
}
