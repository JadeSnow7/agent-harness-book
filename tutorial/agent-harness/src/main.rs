use tutorial_agent_harness::{Config, DEFAULT_INPUT, ReqwestTransport, chat_once, format_response};

fn main() {
    if let Err(error) = run() {
        eprintln!("tutorial-agent-harness: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), tutorial_agent_harness::AppError> {
    let config = Config::from_env().map_err(tutorial_agent_harness::AppError::Config)?;
    let output = chat_once(&config, DEFAULT_INPUT, &ReqwestTransport)?;
    println!(
        "{}",
        format_response(&output).map_err(tutorial_agent_harness::AppError::Protocol)?
    );
    Ok(())
}
