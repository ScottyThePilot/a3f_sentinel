mod admin;
mod general;
mod owner;

use serenity::{
  prelude::*,
  framework::standard::{
    Args, CommandOptions, Reason,
    macros::*
  },
  model::{
    id::UserId,
    channel::Message,
    guild::Member
  }
};

use crate::data::config::ConfigContainer;
use crate::handler::*;
use crate::util::ResultExt;

pub mod groups {
  pub use super::admin::ADMIN_GROUP;
  pub use super::general::GENERAL_GROUP;
  pub use super::owner::OWNER_GROUP;
}

#[check]
#[name = "admin"]
async fn admin_check(ctx: &Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> Result<(), Reason> {
  // Is the user on the list of owners?
  let config = data_get::<ConfigContainer>(&ctx).await;
  let config_lock = config.read().await;
  if config_lock.owners.contains(&msg.author.id) { return Ok(()) };

  if let Ok(member) = msg.member(&ctx).await {
    // Does the user have an administrator role?
    let has_admin_role = member.roles.iter()
      .any(|&role| config_lock.is_admin_role(role));
    if has_admin_role { return Ok(()) };

    // Does the user have the administrator permission?
    let permissions = member.permissions(&ctx).await.unwrap();
    if permissions.administrator() { return Ok(()) };
  };

  Err(Reason::User("Insufficient permissions".to_string()))
}

async fn get_member_from_args(ctx: &Context, msg: &Message, args: &mut Args) -> Option<Member> {
  let member = args.single::<UserId>().ok()?;
  ctx.cache.member(msg.guild_id.unwrap(), member).await
}

async fn react_success(ctx: &Context, msg: &Message) {
  msg.react(&ctx, '\u{2705}').await.report();
}

async fn react_failure(ctx: &Context, msg: &Message) {
  msg.react(&ctx, '\u{274e}').await.report();
}
