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

use crate::config::*;
use super::*;

#[group]
#[commands(ping, emoji_data)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
  ignore!("Failed to send message: {:?}", msg.reply(&ctx, "pong").await);
  Ok(())
}

#[command("emojidata")]
async fn emoji_data(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Ok(emoji) = args.single::<ReactionType>() {
    let emoji = EmojiData::from(emoji).to_string();
    ignore!(msg.reply(&ctx, format!("`{}`", emoji)).await);
  } else {
    react_failure(&ctx, &msg).await;
  };

  Ok(())
}
