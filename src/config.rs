use std::collections::{HashSet, HashMap, BTreeMap};
use std::path::Path;
use std::sync::Arc;

use async_std::fs;
use serenity::{
  prelude::TypeMapKey,
  model::{
    channel::{Reaction, ReactionType},
    id::{
      UserId, RoleId, GuildId,
      ChannelId, MessageId
    }
  }
};

use crate::error::Error;



pub struct ConfigContainer;

impl TypeMapKey for ConfigContainer {
  type Value = Arc<Config>;
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
  pub role_menu_positions: HashMap<EmojiData, String>
}

impl Config {
  #[inline]
  fn to_bytes(&self) -> Result<Vec<u8>, Error> {
    let pretty = ron::ser::PrettyConfig::new()
      .with_indentor("  ".to_owned())
      .with_decimal_floats(true);
    ron::ser::to_string_pretty(self, pretty)
      .map(String::into_bytes)
      .map_err(Error::Ron)
  }

  #[inline]
  fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
    ron::de::from_bytes(bytes)
      .map_err(Error::Ron)
  }

  pub fn is_admin_role(&self, role_id: RoleId) -> bool {
    self.positions.iter()
      .any(|position| position.admin && position.role == role_id)
  }

  pub fn is_role_menu_reaction(&self, react: &Reaction) -> bool {
    self.role_menu == (react.channel_id, react.message_id) &&
    self.role_menu_positions.contains_key(&react.emoji.clone().into())
  }

  pub fn should_grant_position(&self, position: Option<&Position>) -> bool {
    if let Some(position) = position {
      self.role_menu_positions.values()
        .any(|position_name| *position_name == position.name)
    } else {
      true
    }
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

  pub fn get_role_menu_emoji(&self, position_name: &str) -> Option<EmojiData> {
    self.role_menu_positions.iter()
      .find_map(|(emoji, name)| match name == position_name {
        true => Some(emoji.clone()),
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

  /*pub fn get_assignable(&self, assignable_name: &str) -> Option<RoleId> {
    self.assignable.get(assignable_name).copied()
  }*/

  pub async fn open<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
    use async_std::io::ErrorKind;
    match fs::read(path.as_ref()).await {
      Ok(bytes) => Config::from_bytes(&bytes),
      Err(err) if err.kind() == ErrorKind::NotFound => {
        let config = Config::default().to_bytes()?;
        fs::write(path.as_ref(), &config).await?;
        Err("config.ron not found, created default file".into())
      },
      Err(err) => Err(err.into())
    }
  }
}

impl Default for Config {
  #[inline]
  fn default() -> Config {
    Config {
      owners: HashSet::new(),
      token: "X".repeat(59),
      guild: 0.into(),
      default_rank: "Test".to_owned(),
      ranks: vec![
        Rank {
          role: 0.into(),
          name: "Test".to_owned()
        }
      ],
      positions: vec![
        Position {
          role: 0.into(),
          name: "Test".to_owned(),
          ranked: false,
          admin: false
        }
      ],
      assignable: BTreeMap::new(),
      role_menu: (0.into(), 0.into()),
      role_menu_positions: HashMap::new()
    }
  }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EmojiData(String);

impl From<ReactionType> for EmojiData {
  fn from(value: ReactionType) -> EmojiData {
    EmojiData(value.as_data())
  }
}

impl From<EmojiData> for ReactionType {
  fn from(value: EmojiData) -> ReactionType {
    use std::str::FromStr;
    ReactionType::from_str(&value.0).unwrap()
  }
}

impl From<String> for EmojiData {
  fn from(value: String) -> EmojiData {
    EmojiData(value)
  }
}

impl ToString for EmojiData {
  fn to_string(&self) -> String {
    self.0.clone()
  }
}
