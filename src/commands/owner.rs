use std::collections::HashSet;
use std::sync::Arc;

use serenity::{
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::*
  },
  model::{
    id::RoleId,
    channel::Message
  }
};

use crate::config::*;
use crate::handler::*;
use super::*;

#[group]
#[commands(stop, reload, set_rank, promote, demote)]
struct Owner;

#[command]
#[owners_only]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
  ignore!(msg.react(&ctx, '\u{2705}').await);

  let shard_manager = data_get::<ShardManagerContainer>(&ctx).await;
  let mut shard_manager_lock = shard_manager.lock().await;
  shard_manager_lock.shutdown_all().await;

  Ok(())
}

#[command]
#[owners_only]
async fn reload(ctx: &Context, msg: &Message) -> CommandResult {
  match Config::open("config.ron").await {
    Ok(config) => {
      let mut data = ctx.data.write().await;
      data.insert::<ConfigContainer>(Arc::new(config));
      std::mem::drop(data);
      react_success(&ctx, &msg).await;
    },
    Err(err) => {     
      println!("Failed to reload config: {:?}", err);
      react_failure(&ctx, &msg).await;
    }
  };

  Ok(())
}

#[command("setrank")]
#[only_in(guilds)]
#[owners_only]
async fn set_rank(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let config = data_get::<ConfigContainer>(&ctx).await;
  if let Some(member) = get_member_from_args(&ctx, &msg, &mut args).await {
    if let Some(roles) = change_rank_logic(&config, &member, Scheme::Named(args.rest())) {
      match member.edit(&ctx, |edit| edit.roles(roles)).await {
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
#[only_in(guilds)]
#[owners_only]
async fn promote(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let config = data_get::<ConfigContainer>(&ctx).await;
  if let Some(member) = get_member_from_args(&ctx, &msg, &mut args).await {
    if let Some(roles) = change_rank_logic(&config, &member, Scheme::Higher) {
      match member.edit(&ctx, |edit| edit.roles(roles)).await {
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
#[only_in(guilds)]
#[owners_only]
async fn demote(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let config = data_get::<ConfigContainer>(&ctx).await;
  if let Some(member) = get_member_from_args(&ctx, &msg, &mut args).await {
    if let Some(roles) = change_rank_logic(&config, &member, Scheme::Lower) {
      match member.edit(&ctx, |edit| edit.roles(roles)).await {
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

fn change_rank_logic(config: &Config, member: &Member, scheme: Scheme<'_>) -> Option<HashSet<RoleId>> {
  let ranks = config.get_member_ranks(&member.roles);
  let old_rank = *ranks.first()?;
  let new_rank = match scheme {
    Scheme::Lower => config.get_lower_rank(&old_rank.name)?,
    Scheme::Higher => config.get_higher_rank(&old_rank.name)?,
    Scheme::Named(name) => config.get_rank_by_name_loose(name)?
  };

  if old_rank == new_rank { return None };

  let mut roles: HashSet<RoleId> = member.roles.iter().cloned().collect();
  roles.insert(new_rank.role);
  roles.remove(&old_rank.role);

  Some(roles)
}

enum Scheme<'a> {
  Lower,
  Higher,
  Named(&'a str)
}
