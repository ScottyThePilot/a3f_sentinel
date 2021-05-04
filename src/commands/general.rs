use serenity::{
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::*
  },
  model::{
    channel::{Message, ReactionType}
  }
};

use crate::util::ResultExt;
use super::*;

#[group]
#[commands(ping, emoji_data)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
  msg.reply(&ctx, "pong").await.report_with("Failed to send message");
  Ok(())
}

#[command("emojidata")]
async fn emoji_data(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  use singlefile::serde_multi::formats::json;
  if let Ok(emoji) = args.single::<ReactionType>() {
    if let Ok(emoji) = json::to_string(&emoji) {
      msg.reply(&ctx, format!("`{}`", emoji)).await.report();
    } else {
      react_failure(&ctx, &msg).await;
    };
  } else {
    react_failure(&ctx, &msg).await;
  };

  Ok(())
}
