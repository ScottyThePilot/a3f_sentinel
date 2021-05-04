use std::collections::HashSet;

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

use crate::data::config::{Config, ConfigContainer};
use crate::data::persist::PersistContainer;
use crate::handler::*;
use crate::util::ResultExt;
use super::*;

#[group]
#[commands(stop, reload, reset_greets, set_rank, promote, demote)]
struct Owner;

#[command]
#[owners_only]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
  msg.react(&ctx, '\u{2705}').await.report();

  let shard_manager = data_get::<ShardManagerContainer>(&ctx).await;
  let mut shard_manager_lock = shard_manager.lock().await;
  shard_manager_lock.shutdown_all().await;

  Ok(())
}

#[command]
#[owners_only]
async fn reload(ctx: &Context, msg: &Message) -> CommandResult {
  let config = data_get::<ConfigContainer>(&ctx).await;
  let mut config_lock = config.write().await;
  let persist = data_get::<PersistContainer>(&ctx).await;
  let mut persist_lock = persist.write().await;
  match (config_lock.refresh(), persist_lock.refresh()) {
    (Ok(()), Ok(())) => {
      react_success(&ctx, &msg).await;
    },
    (config_result, persist_result) => {     
      config_result.report_with("Failed to reload config");
      persist_result.report_with("Failed to reload persist");
      react_failure(&ctx, &msg).await;
    }
  };

  Ok(())
}

#[command("resetgreets")]
#[only_in(guilds)]
#[owners_only]
async fn reset_greets(ctx: &Context, msg: &Message) -> CommandResult {
  let persist = data_get::<PersistContainer>(&ctx).await;
  let mut persist_lock = persist.write().await;
  
  let members = msg.guild_id.unwrap()
    .members(ctx, None, None).await?;
  let members = members.into_iter()
    .map(|member| member.user.id)
    .collect::<HashSet<UserId>>();
  persist_lock.greeted_users = members;
  match persist_lock.commit() {
    Ok(()) => react_success(&ctx, &msg).await,
    Err(err) => {
      println!("Unable to commit persistence: {:?}", err);
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
  let config_lock = config.read().await;
  if let Some(member) = get_member_from_args(&ctx, &msg, &mut args).await {
    if let Some(roles) = change_rank(&config_lock, &member, Scheme::Named(args.rest())) {
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
  let config_lock = config.read().await;
  if let Some(member) = get_member_from_args(&ctx, &msg, &mut args).await {
    if let Some(roles) = change_rank(&config_lock, &member, Scheme::Higher) {
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
  let config_lock = config.read().await;
  if let Some(member) = get_member_from_args(&ctx, &msg, &mut args).await {
    if let Some(roles) = change_rank(&config_lock, &member, Scheme::Lower) {
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

fn change_rank(config: &Config, member: &Member, scheme: Scheme<'_>) -> Option<HashSet<RoleId>> {
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
