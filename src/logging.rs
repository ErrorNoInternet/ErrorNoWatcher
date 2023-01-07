use colored::*;

pub enum LogMessageType {
    Bot,
    Chat,
    Error,
}

pub fn log_error<T, E: std::fmt::Display>(result: Result<T, E>) {
    match result {
        Ok(_) => (),
        Err(error) => log_message(LogMessageType::Error, &error.to_string()),
    }
}

pub fn log_message(message_type: LogMessageType, message: &String) {
    match message_type {
        LogMessageType::Bot => println!("{} {}", colored_brackets(&"BOT".bold().blue()), message),
        LogMessageType::Chat => println!("{} {}", colored_brackets(&"CHAT".bold().blue()), message),
        LogMessageType::Error => println!(
            "{} {}",
            colored_brackets(&"ERROR".bold().red()),
            message.red()
        ),
    }
}

fn colored_brackets(text: &ColoredString) -> String {
    format!("{}{}{}", "[".bold().yellow(), text, "]".bold().yellow())
}
