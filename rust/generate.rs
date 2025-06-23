use agentic_coding_protocol as acp;
use schemars::generate::SchemaSettings;
use std::fs;

fn main() {
    let settings = SchemaSettings::default().for_serialize();
    let generator = settings.into_generator();
    let mut schema = generator.into_root_schema_for::<acp::Message>();
    {
        let schema = schema.as_object_mut().unwrap();
        schema.remove("title");
    }

    fs::write(
        "./schema.json",
        serde_json::to_string_pretty(&schema).unwrap(),
    )
    .unwrap();
    fs::write(
        "./target/client_requests.json",
        serde_json::to_string_pretty(&acp::CLIENT_METHODS).unwrap(),
    )
    .unwrap();
    fs::write(
        "./target/agent_requests.json",
        serde_json::to_string_pretty(&acp::AGENT_METHODS).unwrap(),
    )
    .unwrap();
}
