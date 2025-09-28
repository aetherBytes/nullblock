use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    pub name: String,
    pub description: String,
    pub url: String,

    #[serde(rename = "preferredTransport", skip_serializing_if = "Option::is_none")]
    pub preferred_transport: Option<TransportProtocol>,

    #[serde(rename = "additionalInterfaces", skip_serializing_if = "Option::is_none")]
    pub additional_interfaces: Option<Vec<AgentInterface>>,

    #[serde(rename = "iconUrl", skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AgentProvider>,

    pub version: String,

    #[serde(rename = "documentationUrl", skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,

    pub capabilities: AgentCapabilities,

    #[serde(rename = "securitySchemes", skip_serializing_if = "Option::is_none")]
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,

    #[serde(rename = "defaultInputModes")]
    pub default_input_modes: Vec<String>,

    #[serde(rename = "defaultOutputModes")]
    pub default_output_modes: Vec<String>,

    pub skills: Vec<AgentSkill>,

    #[serde(rename = "supportsAuthenticatedExtendedCard", skip_serializing_if = "Option::is_none")]
    pub supports_authenticated_extended_card: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<AgentCardSignature>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProvider {
    pub organization: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,

    #[serde(rename = "pushNotifications", skip_serializing_if = "Option::is_none")]
    pub push_notifications: Option<bool>,

    #[serde(rename = "stateTransitionHistory", skip_serializing_if = "Option::is_none")]
    pub state_transition_history: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<AgentExtension>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExtension {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<String>>,
    #[serde(rename = "inputModes", skip_serializing_if = "Option::is_none")]
    pub input_modes: Option<Vec<String>>,
    #[serde(rename = "outputModes", skip_serializing_if = "Option::is_none")]
    pub output_modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInterface {
    pub url: String,
    pub transport: TransportProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportProtocol {
    #[serde(rename = "JSONRPC")]
    JsonRpc,
    #[serde(rename = "GRPC")]
    Grpc,
    #[serde(rename = "HTTP+JSON")]
    HttpJson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCardSignature {
    pub protected: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey {
        #[serde(rename = "in")]
        location: String,
        name: String,
    },
    #[serde(rename = "http")]
    Http {
        scheme: String,
        #[serde(rename = "bearerFormat", skip_serializing_if = "Option::is_none")]
        bearer_format: Option<String>,
    },
    #[serde(rename = "oauth2")]
    OAuth2 {
        flows: serde_json::Value,
    },
    #[serde(rename = "openIdConnect")]
    OpenIdConnect {
        #[serde(rename = "openIdConnectUrl")]
        open_id_connect_url: String,
    },
}

impl Default for AgentCard {
    fn default() -> Self {
        let protocols_base_url = std::env::var("PROTOCOLS_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8001".to_string());

        Self {
            protocol_version: "0.3.0".to_string(),
            name: "NullBlock Protocol Service".to_string(),
            description: "Multi-protocol agent communication service supporting MCP and A2A protocols for intelligent agent workflows".to_string(),
            url: format!("{}/a2a/v1", protocols_base_url),
            preferred_transport: Some(TransportProtocol::JsonRpc),
            additional_interfaces: Some(vec![
                AgentInterface {
                    url: format!("{}/a2a/jsonrpc", protocols_base_url),
                    transport: TransportProtocol::JsonRpc,
                },
                AgentInterface {
                    url: format!("{}/a2a/v1", protocols_base_url),
                    transport: TransportProtocol::HttpJson,
                },
            ]),
            provider: Some(AgentProvider {
                organization: "NullBlock".to_string(),
                url: "https://nullblock.io".to_string(),
            }),
            icon_url: None,
            version: "1.0.0".to_string(),
            documentation_url: Some("https://docs.nullblock.io/protocols".to_string()),
            capabilities: AgentCapabilities {
                streaming: Some(true),
                push_notifications: Some(true),
                state_transition_history: Some(false),
                extensions: None,
            },
            security_schemes: None,
            security: None,
            default_input_modes: vec![
                "application/json".to_string(),
                "text/plain".to_string(),
            ],
            default_output_modes: vec![
                "application/json".to_string(),
                "text/plain".to_string(),
            ],
            skills: vec![
                AgentSkill {
                    id: "task-management".to_string(),
                    name: "Task Management".to_string(),
                    description: "Create, manage, and execute tasks within the NullBlock agent ecosystem".to_string(),
                    tags: vec!["tasks".to_string(), "automation".to_string(), "workflow".to_string()],
                    examples: Some(vec![
                        "Create a new task for data analysis".to_string(),
                        "List all running tasks".to_string(),
                    ]),
                    input_modes: None,
                    output_modes: None,
                    security: None,
                },
                AgentSkill {
                    id: "protocol-routing".to_string(),
                    name: "Protocol Routing".to_string(),
                    description: "Route messages between different agent protocols (MCP, A2A)".to_string(),
                    tags: vec!["routing".to_string(), "protocols".to_string(), "interoperability".to_string()],
                    examples: Some(vec![
                        "Route MCP request to A2A agent".to_string(),
                        "Translate between protocol formats".to_string(),
                    ]),
                    input_modes: None,
                    output_modes: None,
                    security: None,
                },
            ],
            supports_authenticated_extended_card: Some(false),
            signatures: None,
        }
    }
}