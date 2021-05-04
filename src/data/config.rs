use std::collections::{HashSet, BTreeMap};
use std::sync::Arc;

use singlefile::serde_multi::formats::json::Json;
use singlefile::BackendReadonly;
use serenity::{
  prelude::{TypeMapKey, RwLock},
  model::{
    channel::{Reaction, ReactionType},
    id::{
      UserId, RoleId, GuildId,
      ChannelId, MessageId
    }
  }
};

pub type ConfigFile = BackendReadonly<Config, Json>;

pub struct ConfigContainer;

impl TypeMapKey for ConfigContainer {
  type Value = Arc<RwLock<ConfigFile>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
  /// A list of user ids that have absolute authority
  pub owners: HashSet<UserId>,
  /// Token used to sign the bot in
  pub token: String,
  /// The only guild sentinel will respond in
  pub guild: GuildId,
  /// Rank to give to people if they need a rank and have none
  pub default_rank: String,
  /// The rank ladder used in `promote` and `demote`
  pub ranks: Vec<Rank>,
  /// Position roles, only one of which may be had at a time
  pub positions: Vec<Position>,
  /// Roles which may be assigned to users
  pub assignable: BTreeMap<String, RoleId>,
  /// The channel and message id of the role reaction menu
  pub role_menu: (ChannelId, MessageId),
  /// A map determining which emoji grants which position role
  pub role_menu_positions: Vec<RoleMenuPosition>,
  /// Position, which when given through the role menu will trigger a greeting
  pub greetable_positions: HashSet<String>,
  /// Channel to paste greetings into
  pub greeting_channel: ChannelId,
  /// Text to use for a greeting message
  pub greeting: Vec<String>
}

impl Config {
  pub fn is_admin_role(&self, role_id: RoleId) -> bool {
    self.positions.iter()
      .any(|position| position.admin && position.role == role_id)
  }

  pub fn is_role_menu_reaction(&self, react: &Reaction) -> bool {
    self.role_menu == (react.channel_id, react.message_id) &&
    self.role_menu_positions.iter()
      .any(|role_position| role_position.emoji == react.emoji)
  }

  pub fn should_grant_position(&self, position: Option<&Position>) -> bool {
    if let Some(position) = position {
      self.role_menu_positions.iter()
        .any(|role_position| role_position.name == position.name)
    } else {
      true
    }
  }

  pub fn get_role_menu_position(&self, emoji: &ReactionType) -> Option<&RoleMenuPosition> {
    self.role_menu_positions.iter()
      .find(|role_position| role_position.emoji == *emoji)
  }

  pub fn get_rank_by_name_loose(&self, rank_name: &str) -> Option<&Rank> {
    let rank_name = rank_name.to_lowercase();
    self.ranks.iter()
      .find(|rank| rank.name.to_lowercase() == rank_name)
  }

  pub fn get_rank_by_name(&self, rank_name: &str) -> Option<&Rank> {
    self.ranks.iter()
      .find(|rank| rank.name == rank_name)
  }

  pub fn get_member_ranks(&self, roles: &[RoleId]) -> Vec<&Rank> {
    roles.iter()
      .filter_map(|role| {
        self.ranks.iter()
          .find_map(|rank| match rank.role == *role {
            true => Some(rank),
            false => None
          })
      })
      .collect()
  }

  /*pub fn get_position_by_name_loose(&self, position_name: &str) -> Option<&Position> {
    let position_name = position_name.to_lowercase();
    self.positions.iter()
      .find(|position| position.name.to_lowercase() == position_name)
  }*/

  pub fn get_position_by_name(&self, position_name: &str) -> Option<&Position> {
    self.positions.iter()
      .find(|position| position.name == position_name)
  }

  pub fn get_member_positions(&self, roles: &[RoleId]) -> Vec<&Position> {
    roles.iter()
      .filter_map(|role| {
        self.positions.iter()
          .find_map(|position| match position.role == *role {
            true => Some(position),
            false => None
          })
      })
      .collect()
  }

  pub fn get_role_menu_emoji(&self, position_name: &str) -> Option<ReactionType> {
    self.role_menu_positions.iter()
      .find_map(|role_position| match role_position.name == position_name {
        true => Some(role_position.emoji.clone()),
        false => None
      })
  }

  pub fn get_higher_rank(&self, rank_name: &str) -> Option<&Rank> {
    let index = self.ranks.iter()
      .position(|rank| rank.name == rank_name)?;
    self.ranks.get(index + 1)
  }

  pub fn get_lower_rank(&self, rank_name: &str) -> Option<&Rank> {
    let index = self.ranks.iter()
      .position(|rank| rank.name == rank_name)?;
    self.ranks.get(index - 1)
  }

  pub fn get_assignable_loose(&self, assignable_name: &str) -> Option<RoleId> {
    let assignable_name = assignable_name.to_lowercase();
    self.assignable.iter()
      .find_map(|(name, role_id)| match name.to_lowercase() == assignable_name {
        true => Some(role_id.clone()),
        false => None
      })
  }

  pub fn get_greeting(&self) -> String {
    self.greeting.join("\n")
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
  pub name: String,
  pub role: RoleId,
  pub ranked: bool,
  pub admin: bool
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rank {
  pub name: String,
  pub role: RoleId
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleMenuPosition {
  pub emoji: ReactionType,
  pub name: String
}
