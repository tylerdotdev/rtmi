use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Command {
    PrivMsg,
    UserNotice,
    Ping,
    Pong,
    Other(String)
}

#[derive(Debug)]
pub struct IrcMessage {
    pub raw: String,
    pub tags: HashMap<String, String>,
    pub prefix: Option<String>,
    pub command: Option<Command>,
    pub params: Vec<String>
}

pub fn parse(line: &str) -> Option<IrcMessage> {
    let mut message = IrcMessage {
        raw: line.to_string(),
        tags: HashMap::new(),
        prefix: None,
        command: None,
        params: vec![]
    };

    // tags
    let line = if line.chars().next() == Some('@') {
        let (tags, rest) = next_segment(&line[1..]).unwrap();

        let raw_tags = tags.split(";");
        
        for tag in raw_tags {
            if tag.contains("=") {
                let mut pair = tag.split("=");

                &message.tags.insert(pair.next().unwrap().to_string(),
                                    pair.next().unwrap().to_string());
            } else {
                &message.tags.insert(tag.to_string(), "".to_string());
            }
        }

        rest
    } else {
        line
    };

    // prefix
    let line = if line.chars().next() == Some(':') {
        let (prefix, rest) = next_segment(&line[1..]).unwrap();
        message.prefix = Some(prefix.to_string());

        rest
    } else {
        line
    };

    // command
    let line = match next_segment(line) {
        None if line.len() > 0 => {
            message.command = Some(get_command(&line));

            return Some(message);
        }
        None => return None,
        Some((segment, rest)) => {
            message.command = Some(get_command(&segment));

            rest
        }
    };

    // params
    let mut rest = line;
    while !rest.is_empty() {
        if let Some(':') = rest.chars().next() {
            message.params.push(rest[1..].to_string());
            break;
        }

        match next_segment(rest) {
            None => {
                message.params.push(rest.to_string());
                break;
            }
            Some((last, "")) => {
                message.params.push(last.to_string());
                break;
            }
            Some((next, tail)) => {
                message.params.push(next.to_string());
                rest = tail;
            }
        }
    }

    Some(message)
}

fn next_segment(line: &str) -> Option<(&str, &str)> {
    match line.find(' ') {
        Some(n) => {
            let segment = &line[..n];
            let rest = &line[n..].trim_start();
            Some((segment, rest))
        },
        None => None
    }
}

fn get_command(line: &str) -> Command {
    match line {
        "PRIVMSG" => Command::PrivMsg,
        "USERNOTICE" => Command::UserNotice,
        "PING" => Command::Ping,
        "PONG" => Command::Pong,
        _ => Command::Other(line.to_string())
    }
}