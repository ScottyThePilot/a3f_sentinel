use serenity::{
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::*
  },
  model::{
    id::RoleId,
    guild::Member,
    channel::Message
  }
};

use crate::data::config::{Config, ConfigContainer};
use crate::handler::*;
use super::*;

#[group]
#[commands(assign, unassign)]
struct Admin;

#[command]
#[only_in(guilds)]
#[checks(admin)]
async fn assign(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let config = data_get::<ConfigContainer>(&ctx).await;
  let config_lock = config.read().await;
  if let Some(mut member) = get_member_from_args(&ctx, &msg, &mut args).await {
    if let Some(role) = change_assignable(&config_lock, &member, Scheme::Assign(args.rest())) {
      match member.add_role(&ctx, role).await {
        Ok(_) => react_success(&ctx, &msg).await,
        Err(_) => react_failure(&ctx, &msg).await
      };
    } else {
      react_failure(&ctx, &msg).await;
    };
  } else {
    react_failure(&ctx, &msg).await;
  };
  
  Ok(())
}

#[command]
async fn unassign(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let config = data_get::<ConfigContainer>(&ctx).await;
  let config_lock = config.read().await;
  if let Some(mut member) = get_member_from_args(&ctx, &msg, &mut args).await {
    if let Some(role) = change_assignable(&config_lock, &member, Scheme::Unassign(args.rest())) {
      match member.remove_role(&ctx, role).await {
        Ok(_) => react_success(&ctx, &msg).await,
        Err(_) => react_failure(&ctx, &msg).await
      };
    } else {
      react_failure(&ctx, &msg).await;
    };
  } else {
    react_failure(&ctx, &msg).await;
  };
  
  Ok(())
}

fn change_assignable(config: &Config, member: &Member, scheme: Scheme<'_>) -> Option<RoleId> {
  let role = config.get_assignable_loose(scheme.as_str())?;
  match (scheme, member.roles.contains(&role)) {
    (Scheme::Assign(_), true) => None,
    (Scheme::Unassign(_), false) => None,
    _ => Some(role)
  }
}

enum Scheme<'a> {
  Assign(&'a str),
  Unassign(&'a str)
}

impl<'a> Scheme<'a> {
  fn as_str(&self) -> &'a str {
    match self {
      Scheme::Assign(s) | Scheme::Unassign(s) => s
    }
  }
}
