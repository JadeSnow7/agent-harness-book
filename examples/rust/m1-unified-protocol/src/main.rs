//! M1 CLI：通过统一协议完成一次非流式模型调用。

use m1_unified_protocol::{
    DEFAULT_PROMPT, ReqwestTransport, chat_once, format_response, load_config,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("m1-unified-protocol: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), m1_unified_protocol::ProtocolError> {
    let config = load_config()?;
    let mut transport = ReqwestTransport;
    let response = chat_once(DEFAULT_PROMPT, &config, &mut transport)?;
    println!("{}", format_response(&response)?);
    Ok(())
}
