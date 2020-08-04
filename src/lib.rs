mod config;
mod parser;

pub use crate::config::Config;
use crate::parser::{Command, IrcMessage, parse};

use tungstenite::{connect, Message};
use tungstenite::client::AutoStream;
use tungstenite::protocol::WebSocket;
use url::Url;

#[derive(Debug, PartialEq)]
pub enum EventType {
    Message,
    Raid,
    Resub,
    Sub,
    SubGift,
}

#[derive(Debug)]
pub struct Event {
    pub event_type: EventType,
    pub username: String,
    pub moderator: bool,
    pub subscriber: bool,
    pub tier: Option<String>,
    pub message: Option<String>
}

impl From<IrcMessage> for Event {
    fn from(irc: IrcMessage) -> Self {
        let mut event = Event {
            event_type: EventType::Message,
            username: String::new(),
            moderator: false,
            subscriber: false,
            tier: None,
            message: None
        };

        let tags = irc.tags;

        let username = tags["display-name"].clone();
        event.username = username;

        if let Some(cmd) = &irc.command {
            match cmd {
                Command::PrivMsg => event.event_type = EventType::Message,
                Command::UserNotice => {
                    match &tags["msg-id"] {
                        id if id == &"raid".to_string() => event.event_type = EventType::Raid,
                        id if id == &"resub".to_string() => event.event_type = EventType::Resub,
                        id if id == &"sub".to_string() => event.event_type = EventType::Sub,
                        id if id == &"subgift".to_string() => event.event_type = EventType::SubGift,
                        _ => ()
                    }
                    let tier = tags["msg-param-sub-plan"].clone();
                    event.tier = Some(tier);
                },
                _ => ()
            }
        }

        match &tags["mod"] {
            t if t == &"1".to_string() => event.moderator = true,
            f if f == &"0".to_string() => event.moderator = false,
            _ => event.moderator = false
        }

        match &tags["subscriber"] {
            t if t == &"1".to_string() => event.subscriber = true,
            f if f == &"0".to_string() => event.subscriber = false,
            _ => event.subscriber = false
        }

        if irc.params.len() > 1 {
            event.message = Some(irc.params[1].trim_end().to_string());
        }

        event
    }
}

pub struct Client {
    client: Option<WebSocket<AutoStream>>,
    config: Config
}

impl Client {
    pub fn new(config: Config) -> Self {
        Self {
            client: None,
            config
        }
    }

    pub fn connect(&mut self) {
        let (client, _response) =
            connect(Url::parse("ws://irc-ws.chat.twitch.tv:80")
                .unwrap())
                .expect("Failed to connect to IRC");

        self.client = Some(client);

        self.send("CAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership".into());
        self.send(format!("PASS {}", &self.config.token));
        self.send(format!("NICK {}", &self.config.username));
        self.send(format!("JOIN #{}", &self.config.channel));

        println!("Connected to: #{}", &self.config.channel);
    }

    pub fn read_event(&mut self, event_handler: fn(&Event)) {
        if let Some(client) = &mut self.client {
            let message = client.read_message().expect("Error reading message");
            let parsed: IrcMessage = parse(message.to_text().unwrap()).unwrap();

            match parsed.command.as_ref().unwrap() {
                Command::PrivMsg |
                Command::UserNotice => event_handler(&Event::from(parsed)),
                Command::Ping => {
                    self.client
                        .as_mut()
                        .unwrap()
                        .write_message(Message::Text("PONG :tmi.twitch.tv".into()))
                        .unwrap();
                },
                _ => ()
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    pub fn send(&mut self, message: String) {
        if let Some(client) = &mut self.client {
            client
                .write_message(Message::Text(message))
                .unwrap();
        }
    }
}
