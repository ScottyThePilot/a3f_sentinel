use std::collections::HashSet;
use std::sync::Arc;

use serenity::{
  prelude::*,
  client::bridge::gateway::{
    ShardManager, GatewayIntents
  },
  framework::standard::StandardFramework,
  http::Http,
  model::{
    id::{RoleId, GuildId},
    guild::{Member},
    channel::{Reaction},
    gateway::Ready
  }
};

use crate::error::Error;
use crate::config::*;
use crate::commands::groups::*;



pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
  async fn ready(&self, _: Context, ready: Ready) {
    println!("{} is connected", ready.user.name);
  }

  async fn cache_ready(&self, ctx: Context, _: Vec<GuildId>) {
    let config = data_get::<ConfigContainer>(&ctx).await;
    let (channel_id, message_id) = config.role_menu;
    if let Ok(message) = ctx.http.get_message(channel_id.into(), message_id.into()).await {
      for emoji in config.role_menu_positions.keys() {
        ignore!(message.react(&ctx, emoji.clone()).await);
      };
    } else {
      println!("Error: Couldn't find role menu message");
    };
  }

  async fn reaction_add(&self, ctx: Context, react: Reaction) {
    let config = data_get::<ConfigContainer>(&ctx).await;

    // Filter to reactions in the server on the reaction menu message
    if config.is_role_menu_reaction(&react) {
      let user_id = react.user_id.unwrap();
      let guild_id = react.guild_id.unwrap();

      if let Some(member) = ctx.cache.member(guild_id, user_id).await {
        if member.user.bot { return; }; // Ignore reactions from bots
        maybe_grant_position(ctx, config, member, react).await;
      };
    };
  }
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}

pub async fn launch() -> Result<(), Error> {
  let config = Config::open("config.ron").await?;
  let http = Http::new_with_token(&config.token);
  let me = http.get_current_user().await?.id;
  let framework = StandardFramework::new()
    .configure(|cfg| {
      cfg
        .on_mention(Some(me))
        .owners(config.owners.clone())
        .prefix("$")
    })
    .group(&OWNER_GROUP)
    .group(&ADMIN_GROUP)
    .group(&GENERAL_GROUP);

  let mut client = Client::builder(&config.token)
    .event_handler(Handler)
    .framework(framework)
    .intents(intents())
    .await?;
  
  let mut data = client.data.write().await;
  data.insert::<ConfigContainer>(Arc::new(config));
  data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
  std::mem::drop(data);

  let shard_manager = client.shard_manager.clone();
  tokio::spawn(async move {
    tokio::signal::ctrl_c().await.unwrap();
    shard_manager.lock().await.shutdown_all().await;
  });

  client.start().await?;
  Ok(())
}



#[inline]
pub async fn data_get<K>(ctx: &Context) -> K::Value
where K: TypeMapKey, K::Value: Clone {
  ctx.data.read().await.get::<K>().unwrap().clone()
}

async fn maybe_grant_position(ctx: Context, config: Arc<Config>, member: Member, react: Reaction) {
  let positions = config.get_member_positions(&member.roles);
  let current_position = positions.first().copied();

  if config.should_grant_position(current_position) {
    // User has no position role or has a position on the role menu
    let mut roles: HashSet<RoleId> = member.roles.iter().cloned().collect();

    // Give user new position
    let emoji = EmojiData::from(react.emoji.clone());
    let new_position_name = config.role_menu_positions[&emoji].as_str();
    let new_position = config.get_position_by_name(new_position_name).unwrap();
    roles.insert(new_position.role);

    // Remove user's old position(s)
    for &old_position in positions.iter() {
      if old_position != new_position {
        roles.remove(&old_position.role);
      };
    };

    // Assign users ranks if they are supposed to have them
    let ranks = config.get_member_ranks(&member.roles);
    if new_position.ranked && ranks.is_empty() {
      // User should have a rank, has no ranks
      let default_rank = config.get_rank_by_name(&config.default_rank).unwrap();
      roles.insert(default_rank.role);
    } else if new_position.ranked && ranks.len() > 1 {
      // User should have a rank, has more than 1 rank
      for &old_rank in ranks.iter().skip(1) {
        roles.remove(&old_rank.role);
      };
    } else if !new_position.ranked && !ranks.is_empty() {
      // User should not have a rank, has at least 1 rank
      for old_rank in ranks {
        roles.remove(&old_rank.role);
      };
    };

    ignore!("Couldn't edit roles: {:?}", member.edit(&ctx, |edit| edit.roles(roles)).await);

    // Do nothing else if the user tried to give themselves a role they already have
    if Some(new_position) == current_position { return };

    // Delete their other reactions
    let guild_channel = ctx.cache.guild_channel(react.channel_id).await.unwrap();
    for &old_position in positions.iter() {
      if old_position != new_position {
        if let Some(emoji) = config.get_role_menu_emoji(&old_position.name) {
          ignore!(guild_channel.delete_reaction(&ctx, react.message_id, Some(member.user.id), emoji).await);
        };
      };
    };
  } else {
    // User has a position not on the role menu; don't change their roles
    let guild_channel = ctx.cache.guild_channel(react.channel_id).await.unwrap();
    ignore!(guild_channel.delete_reaction(&ctx, react.message_id, Some(member.user.id), react.emoji).await);
  };
}

#[inline]
fn intents() -> GatewayIntents {
  GatewayIntents::GUILDS |
  GatewayIntents::GUILD_MEMBERS |
  GatewayIntents::GUILD_BANS |
  GatewayIntents::GUILD_EMOJIS |
  GatewayIntents::GUILD_PRESENCES |
  GatewayIntents::GUILD_MESSAGES |
  GatewayIntents::GUILD_MESSAGE_REACTIONS
}
