use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

pub mod storage;

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedContainer {
    docker_id: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    containers: HashMap<String, SavedContainer>,
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Conatiner with name \"{0}\" already exists")]
    ContainerAlreadyExists(String),
}

impl State {
    pub fn container(&self, name: &str) -> Option<&SavedContainer> {
        self.containers.get(name)
    }

    pub fn add_container(
        &mut self,
        name: String,
        container: SavedContainer,
    ) -> Result<(), StateError> {
        if self.container(&name).is_some() {
            return Err(StateError::ContainerAlreadyExists(name));
        }

        self.containers.insert(name, container);

        Ok(())
    }
}

impl SavedContainer {
    pub fn docker_id(&self) -> &str {
        &self.docker_id
    }

    pub fn from_docker_id(docker_id: String) -> SavedContainer {
        SavedContainer { docker_id }
    }
}
