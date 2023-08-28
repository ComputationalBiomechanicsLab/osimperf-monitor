use serde::{Serialize, Deserialize, de::DeserializeOwned};

pub trait NodeFile : Serialize + DeserializeOwned {
    type State;

    fn state(&self) -> &Self::State;

    fn mut_state(&self) -> &mut Self::State;

    fn try_write(&mut self, state: &Self::State) -> anyhow::Result<bool> {
        todo!()
    }

    fn try_read(&mut self, state: &mut State) -> anyhow::Result<bool> {
        todo!()
    }

    fn try_open() -> anyhow::Result<Option<Self>> {
        todo!()
    }
}
