use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use reqwest;
use serde_derive::{Deserialize, Serialize};
use serenity::command;
use serenity::utils::MessageBuilder;

lazy_static! {
    static ref CODE: Regex =
        Regex::new(r".+\n+\x60{3}(rust)\n([\s\S]*?)\x60{3}").expect("Compile regex");
}

#[derive(Debug, Serialize)]
pub struct Request {
    backtrace: bool,
    channel: String,
    #[serde(rename = "crateType")]
    crate_type: String,
    edition: String,
    mode: String,
    tests: bool,
    code: String,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub stderr: String,
    pub stdout: String,
    pub success: bool,
}

// ?repl rust
command!(command(_context, message) {
    let caps = if let Some(caps) = CODE.captures(&message.content) {
        caps
    } else {
        message.reply("Couldn't parse your code. Make sure you wrap it in codeblocks with ```rust")?;
        return Ok(());
    };

    // build request payload
    let payload = Request {
        backtrace: false,
        channel: "nightly".into(),
        crate_type: "bin".into(),
        edition: "2018".into(),
        mode: "release".into(),
        tests: false,
        code: caps[2].into(),
    };
    debug!("Rust payload: {:?}", payload);

    // make request to the playground
    let client = reqwest::Client::new();
    let mut res = match client.post("https://play.integer32.com/execute").json(&payload).send() {
        Ok(res) => res,
        Err(e) => {
            debug!("Error: {:#?}", e);
            message.reply("There was an issue sending the code to the REPL.")?;
            return Ok(());
        }
    };

    // deserialize json response into struct
    let json: Response = match res.json() {
        Ok(json) => json,
        Err(e) => {
            debug!("Error: {:#?}", e);
            message.reply("There was an issue with the response from the REPL.")?;
            return Ok(());
        }
    };
    debug!("Rust response: {:?}", json);

    // reply to user
    let message_builder = if json.success {
        MessageBuilder::new()
            .mention(&message.author)
            .push(" ")
            .push("here's the output:")
            .push_codeblock(json.stdout, Some("rust"))
            .build()
    } else {
        MessageBuilder::new()
            .mention(&message.author)
            .push(" ")
            .push("your compilation failed... yikes...")
            .push_codeblock(json.stderr, Some("rust"))
            .build()
    };
    message.channel_id.say(&message_builder)?;
});
