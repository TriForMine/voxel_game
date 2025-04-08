/// The Discord configuration
pub mod config;
/// The state that holds the Discord activity
pub mod state;

use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{log::prelude::*, prelude::*};
use discord_presence::{models::ActivityTimestamps, Client as DiscordClient, Event};

pub use config::{RPCConfig, RPCPlugin};
pub use state::ActivityState;

/// A wrapper around the internal [`discord_presence::Client`] struct that implements [`bevy::prelude::Resource`]
#[derive(Resource, derive_more::Deref, derive_more::DerefMut)]
pub struct Client(DiscordClient);

impl Client {
    /// Instantiates a [`Client`] struct
    ///
    /// Wraps the internal [`discord_presence::Client`] struct
    pub fn new(client_id: u64) -> Self {
        Client(DiscordClient::new(client_id))
    }
}

/// Implements the Bevy plugin trait
impl Plugin for RPCPlugin {
    fn build(&self, app: &mut App) {
        let client_config = self.config;

        app.add_systems(Startup, startup_client);
        app.add_systems(Update, check_activity_changed);
        debug!("Added RPCPlugin systems");

        app.insert_resource::<RPCConfig>(client_config);
        app.init_resource::<ActivityState>();
        app.insert_resource::<Client>(Client::new(client_config.app_id));
        debug!("Initialized RPCPlugin resources");
    }

    fn name(&self) -> &str {
        "Discord Presence"
    }
}

/// Initializes the client and starts it running
fn startup_client(
    mut activity: ResMut<ActivityState>,
    mut client: ResMut<Client>,
    config: Res<RPCConfig>,
) {
    use quork::traits::list::ListVariants;

    if config.show_time {
        activity.timestamps = Some(ActivityTimestamps {
            start: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time has gone backwards")
                    .as_secs(),
            ),
            end: None,
        });
    }

    for event in Event::VARIANTS {
        client
            .on_event(event, {
                let events = activity.events.clone();

                move |_| {
                    events.lock().0.push_back(event);
                    debug!("Added event: {:?}", event);
                }
            })
            .persist();
    }

    client.start();
    debug!("Client has started");
}

/// Runs whenever the activity has been changed, and at startup
fn check_activity_changed(activity: Res<ActivityState>, mut client: ResMut<Client>) {
    if activity.is_changed() {
        let res = client.set_activity(|_| activity.clone().into());

        if let Err(err) = res {
            error!("Failed to set presence: {}", err);
        }
    }
}
