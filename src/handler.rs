use std::collections::HashSet;
use std::sync::Arc;

use singlefile::serde_multi::formats::json::Json;
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
    gateway::Ready,
    event::ResumedEvent,
    misc::Mention
  }
};

use crate::commands::groups::*;
use crate::data::config::{Config, ConfigContainer, ConfigFile};
use crate::data::persist::{PersistContainer, PersistFile};
use crate::error::Error;
use crate::util::ResultExt;



pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
  async fn ready(&self, _: Context, ready: Ready) {
    println!("Bot {} ({}) is connected", ready.user.name, ready.user.id);
  }

  async fn resume(&self, _: Context, _: ResumedEvent) {
    println!("Bot resumed");
  }

  async fn cache_ready(&self, ctx: Context, _: Vec<GuildId>) {
    let config = data_get::<ConfigContainer>(&ctx).await;
    let config_lock = config.read().await;
    let (channel_id, message_id) = config_lock.role_menu;
    if let Ok(message) = ctx.http.get_message(channel_id.into(), message_id.into()).await {
      for role_position in config_lock.role_menu_positions.iter() {
        message.react(&ctx, role_position.emoji.clone()).await.report();
      };
    } else {
      println!("Error: Couldn't find role menu message");
    };
  }

  async fn guild_unavailable(&self, ctx: Context, guild_id: GuildId) {
    let guild_name = ctx.cache.guild(guild_id).await
      .map_or("?".to_owned(), |g| g.name);
    println!("Guild {} ({}) is unavailable", guild_name, guild_id);
  }

  async fn reaction_add(&self, ctx: Context, react: Reaction) {
    let config = data_get::<ConfigContainer>(&ctx).await;
    let config_lock = config.read().await;

    // Filter to reactions in the server on the reaction menu message
    if config_lock.is_role_menu_reaction(&react) {
      let user_id = react.user_id.unwrap();
      let guild_id = react.guild_id.unwrap();

      if let Some(member) = ctx.cache.member(guild_id, user_id).await {
        if member.user.bot { return; }; // Ignore reactions from bots
        maybe_grant_position(ctx, &config_lock, member, react).await;
      };
    };
  }
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}

pub async fn launch() -> Result<(), Error> {
  let config = ConfigFile::open("config.json", Json)?;
  let persist = PersistFile::create_or_default("persist.json", Json)?;
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
  data.insert::<ConfigContainer>(Arc::new(RwLock::new(config)));
  data.insert::<PersistContainer>(Arc::new(RwLock::new(persist)));
  data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
  std::mem::drop(data);

  let shard_manager = Arc::clone(&client.shard_manager);
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

async fn maybe_grant_position(ctx: Context, config: &Config, member: Member, react: Reaction) {
  let positions = config.get_member_positions(&member.roles);
  let current_position = positions.first().copied();

  if config.should_grant_position(current_position) {
    // User has no position role or has a position on the role menu
    let mut roles: HashSet<RoleId> = member.roles.iter().cloned().collect();

    // Give user new position
    let new_position_name = config.get_role_menu_position(&react.emoji).unwrap().name.as_str();
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

    member.edit(&ctx, |edit| edit.roles(roles)).await.report_with("Couldn't edit roles");

    // Do nothing else if the user tried to give themselves a role they already have
    if Some(new_position) == current_position { return };

    // Delete their other reactions
    let guild_channel = ctx.cache.guild_channel(react.channel_id).await.unwrap();
    for &old_position in positions.iter() {
      if old_position != new_position {
        if let Some(emoji) = config.get_role_menu_emoji(&old_position.name) {
          guild_channel.delete_reaction(&ctx, react.message_id, Some(member.user.id), emoji).await.report();
        };
      };
    };

    // Send a greeting in the greeting channel if the correct criteria matches
    let persist = data_get::<PersistContainer>(&ctx).await;
    let mut persist_lock = persist.write().await;
    if persist_lock.should_greet(member.user.id) {
      if config.greetable_positions.contains(&new_position.name) {
        let mention = Mention::from(member.user.id).to_string();
        let greeting = config.get_greeting().replace("{mention}", &mention);
        config.greeting_channel.say(&ctx, greeting).await.report_with("Failed to send greeting");
        persist_lock.register_greeted(member.user.id);
        persist_lock.commit().report_with("Failed to commit config");
      };
    };
  } else {
    // User has a position not on the role menu; don't change their roles
    let guild_channel = ctx.cache.guild_channel(react.channel_id).await.unwrap();
    guild_channel.delete_reaction(&ctx, react.message_id, Some(member.user.id), react.emoji).await.report();
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
