use std::path::PathBuf;

use super::State;

pub struct StateFile {
    path: PathBuf,
    state: State,
}

impl StateFile {
    pub fn new(path: PathBuf) -> Self {
        let raw_state = std::fs::read_to_string(path.clone()).unwrap_or_else(|_| String::new());
        let state = serde_json::from_str(&raw_state).unwrap_or(State::default());

        StateFile { path, state }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

impl Drop for StateFile {
    fn drop(&mut self) {
        std::fs::write(
            &self.path,
            serde_json::to_string_pretty(&self.state).expect("Failed to serialize state"),
        )
        .expect("Failed to write state");
    }
}
