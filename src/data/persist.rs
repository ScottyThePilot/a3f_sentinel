use std::collections::HashSet;
use std::sync::Arc;

use singlefile::serde_multi::formats::json::Json;
use singlefile::BackendWritable;
use serenity::{
  prelude::{TypeMapKey, RwLock},
  model::id::UserId
};

pub type PersistFile = BackendWritable<Persist, Json>;

pub struct PersistContainer;

impl TypeMapKey for PersistContainer {
  type Value = Arc<RwLock<PersistFile>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Persist {
  pub greeted_users: HashSet<UserId>,
}

impl Persist {
  pub fn should_greet(&self, user_id: UserId) -> bool {
    !self.greeted_users.contains(&user_id)
  }

  pub fn register_greeted(&mut self, user_id: UserId) -> bool {
    self.greeted_users.insert(user_id)
  }
}

impl Default for Persist {
  fn default() -> Persist {
    Persist {
      greeted_users: HashSet::new()
    }
  }
}
