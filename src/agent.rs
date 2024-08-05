use bsky_sdk::api::app::bsky::actor::defs::{
    MutedWord, MutedWordData, Preferences, PreferencesItem,
};
use bsky_sdk::api::types::string::AtIdentifier;
use bsky_sdk::api::types::Union;
use bsky_sdk::BskyAgent;
use ipld_core::ipld::Ipld;
use std::str::FromStr;

pub type GetAgentResult = Result<BskyAgent, Box<dyn std::error::Error>>;
pub type MuteActorResult =
    bsky_sdk::api::xrpc::Result<(), bsky_sdk::api::app::bsky::graph::mute_actor::Error>;
pub type UnmuteActorResult =
    bsky_sdk::api::xrpc::Result<(), bsky_sdk::api::app::bsky::graph::unmute_actor::Error>;

pub async fn get_agent(username: &str, password: &str) -> GetAgentResult {
    let agent: BskyAgent = BskyAgent::builder().build().await?;
    agent.login(username, password).await?;
    Ok(agent)
}

pub async fn mute_actor(agent: &BskyAgent, actor: &str) -> MuteActorResult {
    use bsky_sdk::api::app::bsky::graph::mute_actor::{Input, InputData};
    agent
        .api
        .app
        .bsky
        .graph
        .mute_actor(Input {
            data: InputData {
                actor: AtIdentifier::from_str(actor).unwrap(),
            },
            extra_data: Ipld::Null,
        })
        .await
}

pub async fn unmute_actor(agent: &BskyAgent, actor: &str) -> UnmuteActorResult {
    use bsky_sdk::api::app::bsky::graph::unmute_actor::{Input, InputData};
    agent
        .api
        .app
        .bsky
        .graph
        .unmute_actor(Input {
            data: InputData {
                actor: AtIdentifier::from_str(actor).unwrap(),
            },
            extra_data: Ipld::Null,
        })
        .await
}

pub async fn get_preferences(agent: &BskyAgent) -> Preferences {
    use bsky_sdk::api::app::bsky::actor::get_preferences::{Parameters, ParametersData};
    let res = agent
        .api
        .app
        .bsky
        .actor
        .get_preferences(Parameters {
            data: ParametersData {},
            extra_data: Ipld::Null,
        })
        .await;

    res.unwrap().preferences.clone()
}

pub async fn put_preferences(agent: &BskyAgent, preference: Preferences) {
    use bsky_sdk::api::app::bsky::actor::put_preferences::{Input, InputData};
    let res = agent
        .api
        .app
        .bsky
        .actor
        .put_preferences(Input {
            data: InputData {
                preferences: preference,
            },
            extra_data: Ipld::Null,
        })
        .await;
}

pub async fn unmute_actor_by_handle(agent: &BskyAgent, actor_handle: &str) -> UnmuteActorResult {
    use bsky_sdk::api::app::bsky::graph::unmute_actor::{Input, InputData};
    agent
        .api
        .app
        .bsky
        .graph
        .unmute_actor(Input {
            data: InputData {
                actor: AtIdentifier::Handle(actor_handle.parse().unwrap()),
            },
            extra_data: Ipld::Null,
        })
        .await
}

pub async fn add_mute_word_to_pref(agent: &BskyAgent, mute_word: String) {
    let mut preferences = get_preferences(&agent).await;
    for mut preference in &mut preferences {
        match preference {
            Union::Refs(ref mut preferenceItem) => match preferenceItem {
                PreferencesItem::MutedWordsPref(ref mut mute_words_pref) => {
                    let word = MutedWord {
                        data: MutedWordData {
                            targets: vec!["tag".to_string(), "content".to_string()],
                            value: mute_word.clone(),
                        },
                        extra_data: Ipld::Null,
                    };
                    mute_words_pref.items.push(word);
                }
                _ => {}
            },
            Union::Unknown(b) => {}
        }
    }
    put_preferences(&agent, preferences).await;
}

pub async fn remove_mute_word_from_pref(agent: &BskyAgent, mute_word: String) {
    let mut preferences = get_preferences(&agent).await;
    for mut preference in &mut preferences {
        match preference {
            Union::Refs(ref mut preferenceItem) => match preferenceItem {
                PreferencesItem::MutedWordsPref(ref mut mute_words_pref) => {
                    mute_words_pref.items.retain(|word| word.value != mute_word);
                }
                _ => {}
            },
            Union::Unknown(b) => {}
        }
    }
    put_preferences(&agent, preferences).await;
}
