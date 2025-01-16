use anyhow::Context as AContext;
use poise::serenity_prelude::{self as serenity, ChannelId};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Default, Debug)]
struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;

type Context<'a> = poise::Context<'a, Data, Error>;

pub struct TaskNotification {
    channel: u64,
    message: String,
}

async fn scrap_thread(
    mut task_receiver: UnboundedReceiver<TaskNotification>,
    ctx: serenity::Context,
) -> Result<(), Error> {
    ChannelId::from(1329172988486221897).say(&ctx.http, "lol hi").await?;
    while let Some(notification) = task_receiver.recv().await {
        if let Err(why) = ChannelId::from(notification.channel)
            .say(&ctx.http, &notification.message)
            .await
        {
            tracing::error!("{why:?}");
        }
    }

    Ok(())
}

pub async fn start_thread(task_receiver: UnboundedReceiver<TaskNotification>) -> anyhow::Result<()> {
    tokio::spawn(async move {
        tracing::error!("discord token asda");

        let token = std::env::var("DISCORD_TOKEN")
            .context("missing DISCORD_TOKEN")
            .expect("lol");
        let intents = serenity::GatewayIntents::non_privileged();

        tracing::error!("discord token {token:?}");

        let framework = poise::Framework::<Data, Error>::builder()
            .options(poise::FrameworkOptions {
                commands: vec![],
                ..Default::default()
            })
            .setup(move |ctx, _ready, framework| {
                Box::pin(async move {
                    tokio::spawn(scrap_thread(task_receiver, ctx.clone()));

                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                    Ok(Data {})
                })
            })
            .build();

        let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;

        client.unwrap().start().await.unwrap();
    });
    Ok(())
}
