use crate::State;
use azalea::prelude::*;

#[derive(PartialEq, PartialOrd)]
pub enum Command {
    Location,
    Goto,
    Stop,
    Unknown,
}

pub fn process_command(command: &String, _client: &Client, state: &mut State) -> String {
    let check_command = |command: &mut Command, segment: &String| {
        match command {
            Command::Location => return format!("{} is somewhere", segment),
            Command::Goto => {
                if state.final_target.lock().unwrap().is_some()
                    && state.final_target.lock().unwrap().clone().unwrap().len() == 3
                {
                    *command = Command::Unknown;
                    let coordinates =
                        (*state.final_target.lock().unwrap().clone().unwrap()).to_vec();
                    return format!(
                        "I am now going to {} {} {}...",
                        coordinates[0], coordinates[1], coordinates[2]
                    );
                }

                if state.final_target.lock().unwrap().is_none() {
                    *state.final_target.lock().unwrap() = Some(Vec::new());
                };
                let mut new_coordinates = state.final_target.lock().unwrap().clone().unwrap();
                new_coordinates.push(segment.parse().unwrap_or(0));
                *state.final_target.lock().unwrap() = Some(new_coordinates);

                return "".to_string();
            }
            Command::Stop => {
                *state.final_target.lock().unwrap() = None;

                *command = Command::Unknown;
                return "I am no longer doing anything".to_string();
            }
            _ => {
                *command = Command::Unknown;
                return "".to_string();
            }
        };
    };

    let segments: Vec<String> = command
        .split(" ")
        .map(|segment| segment.to_string())
        .collect();
    if segments.len() <= 0 {
        return "Hmm... I was unable to parse your command!".to_string();
    };

    let mut command = Command::Unknown;
    for (_index, segment) in segments.iter().enumerate() {
        match segment.to_lowercase().as_str() {
            "location" => command = Command::Location,
            "goto" => command = Command::Goto,
            "stop" => command = Command::Stop,
            _ => {
                let return_value = check_command(&mut command, &segment);
                if !return_value.is_empty() {
                    return return_value;
                }
            }
        };
    }
    if command != Command::Unknown {
        let return_value = check_command(&mut command, &"".to_string());
        if !return_value.is_empty() {
            return return_value;
        }
    }

    "Sorry, I don't know what you mean...".to_string()
}
