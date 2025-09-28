use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub status: TaskStatus,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
    pub artifacts: Vec<Artifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: MessageContent,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text {
        text: String,
    },
    #[serde(rename = "image")]
    Image {
        #[serde(rename = "imageUrl")]
        image_url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt_text: Option<String>,
    },
    #[serde(rename = "multipart")]
    Multipart {
        parts: Vec<MessageContent>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub data: String, // Base64 encoded data
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
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
    pub fn new(messages: Vec<Message>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            messages,
            artifacts: vec![],
            metadata: None,
        }
    }

    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
        self.updated_at = Utc::now();
    }
}

impl Message {
    pub fn new_text(role: MessageRole, text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content: MessageContent::Text { text },
            timestamp: Utc::now(),
            metadata: None,
        }
    }

    pub fn new_user_text(text: String) -> Self {
        Self::new_text(MessageRole::User, text)
    }

    pub fn new_assistant_text(text: String) -> Self {
        Self::new_text(MessageRole::Assistant, text)
    }
}

impl Artifact {
    pub fn new(name: String, content_type: String, data: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            content_type,
            data,
            created_at: Utc::now(),
            metadata: None,
        }
    }
}