use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    #[serde(rename = "contextId")]
    pub context_id: String,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<Message>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<Artifact>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub state: TaskState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskState {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Canceled,
    Failed,
    Rejected,
    AuthRequired,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub role: MessageRole,
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "referenceTaskIds")]
    pub reference_task_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "taskId")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "contextId")]
    pub context_id: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "agent")]
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Part {
    Text {
        text: String,
    },
    File {
        file: FileBase,
    },
    Data {
        data: serde_json::Value,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FileBase {
    Bytes {
        name: String,
        bytes: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    Uri {
        name: String,
        uri: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotificationConfig {
    pub id: String,
    #[serde(rename = "taskId")]
    pub task_id: String,
    pub webhook_url: String,
    pub events: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
}

impl Task {
    pub fn new(initial_message: Option<Message>) -> Self {
        let context_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        Self {
            id: Uuid::new_v4().to_string(),
            context_id,
            status: TaskStatus {
                state: TaskState::Submitted,
                message: None,
                timestamp: Some(now),
            },
            history: initial_message.map(|msg| vec![msg]),
            artifacts: None,
            metadata: None,
            kind: "task".to_string(),
        }
    }

    pub fn update_status(&mut self, state: TaskState, message: Option<Message>) {
        self.status = TaskStatus {
            state,
            message,
            timestamp: Some(Utc::now().to_rfc3339()),
        };
    }

    pub fn add_message(&mut self, message: Message) {
        if let Some(ref mut history) = self.history {
            history.push(message);
        } else {
            self.history = Some(vec![message]);
        }
    }

    pub fn add_artifact(&mut self, artifact: Artifact) {
        if let Some(ref mut artifacts) = self.artifacts {
            artifacts.push(artifact);
        } else {
            self.artifacts = Some(vec![artifact]);
        }
    }
}

impl Message {
    pub fn new_text(role: MessageRole, text: String) -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            role,
            parts: vec![Part::Text { text }],
            metadata: None,
            extensions: None,
            reference_task_ids: None,
            task_id: None,
            context_id: None,
            kind: "message".to_string(),
        }
    }

    pub fn new_user_text(text: String) -> Self {
        Self::new_text(MessageRole::User, text)
    }

    pub fn new_agent_text(text: String) -> Self {
        Self::new_text(MessageRole::Agent, text)
    }
}

impl Artifact {
    pub fn new(parts: Vec<Part>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            parts,
            metadata: None,
        }
    }

    pub fn new_text(text: String) -> Self {
        Self::new(vec![Part::Text { text }])
    }
}

impl Part {
    pub fn text(text: String) -> Self {
        Part::Text { text }
    }

    pub fn file_bytes(name: String, bytes: String, mime_type: String) -> Self {
        Part::File {
            file: FileBase::Bytes {
                name,
                bytes,
                mime_type,
            },
        }
    }

    pub fn file_uri(name: String, uri: String, mime_type: String) -> Self {
        Part::File {
            file: FileBase::Uri {
                name,
                uri,
                mime_type,
            },
        }
    }

    pub fn data(data: serde_json::Value, mime_type: String) -> Self {
        Part::Data { data, mime_type }
    }
}
