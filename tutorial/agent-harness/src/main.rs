use tutorial_agent_harness::{AppError, Config, DEFAULT_INPUT, ReqwestTransport, chat_once};

fn main() {
    if let Err(error) = run() {
        eprintln!("tutorial-agent-harness: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let config = Config::from_env().map_err(AppError::Config)?;
    let text = chat_once(&config, DEFAULT_INPUT, &ReqwestTransport)?;
    println!("{text}");
    Ok(())
}
