use serde::{Deserialize, Serialize};
use std::fs;

use crate::error::ContentError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub themes: Vec<TemplateTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateTheme {
    pub name: String,
    pub variants: Vec<TemplateVariant>,
    pub placeholders: TemplatePlaceholders,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariant {
    pub template: String,
    pub tags: Vec<String>,
    pub requires_image: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePlaceholders {
    #[serde(default)]
    pub insights: Vec<String>,
    #[serde(default)]
    pub reminders: Vec<String>,
    #[serde(default)]
    pub taglines: Vec<String>,
    #[serde(default)]
    pub statements: Vec<String>,
    #[serde(default)]
    pub punchlines: Vec<String>,
    #[serde(default)]
    pub questions: Vec<String>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub milestones: Vec<String>,
}

pub struct TemplateLoader;

impl TemplateLoader {
    pub fn load_from_json(path: &str) -> Result<TemplateConfig, ContentError> {
        let content = fs::read_to_string(path).map_err(|e| {
            ContentError::GenerationError(format!("Failed to read template file: {}", e))
        })?;
        let config: TemplateConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn get_template_for_theme(config: &TemplateConfig, theme: &str) -> Option<TemplateTheme> {
        config
            .themes
            .iter()
            .find(|t| t.name == theme)
            .cloned()
    }

    pub fn seed_default_templates() -> TemplateConfig {
        TemplateConfig {
            themes: vec![
                TemplateTheme {
                    name: "morning_insight".into(),
                    variants: vec![
                        TemplateVariant {
                            template: "{insight}\n\n{reminder}".into(),
                            tags: vec!["infrastructure".into(), "morning".into()],
                            requires_image: false,
                        },
                        TemplateVariant {
                            template: "{insight}\n\n{tagline}".into(),
                            tags: vec!["protocol".into(), "morning".into()],
                            requires_image: true,
                        },
                    ],
                    placeholders: TemplatePlaceholders {
                        insights: vec![
                            "Protocols don't sleep. Neither should your infrastructure.".into(),
                            "The best agents are the ones you forget are running.".into(),
                        ],
                        reminders: vec![
                            "Ship protocols, not promises.".into(),
                            "Build for compounding, not for applause.".into(),
                        ],
                        taglines: vec![
                            "The void where agentic flows connect.".into(),
                            "Picks and shovels for the agentic gold rush.".into(),
                        ],
                        ..Default::default()
                    },
                },
                TemplateTheme {
                    name: "eerie_fun".into(),
                    variants: vec![
                        TemplateVariant {
                            template: "{statement}\n\n{punchline}".into(),
                            tags: vec!["darkhumor".into(), "ai".into()],
                            requires_image: true,
                        },
                    ],
                    placeholders: TemplatePlaceholders {
                        statements: vec![
                            "Your agents don't sleep. Neither does the inevitable.".into(),
                        ],
                        punchlines: vec![
                            "But hey, at least the uptime is great.".into(),
                        ],
                        ..Default::default()
                    },
                },
                TemplateTheme {
                    name: "community".into(),
                    variants: vec![
                        TemplateVariant {
                            template: "{question}".into(),
                            tags: vec!["community".into(), "engagement".into()],
                            requires_image: false,
                        },
                    ],
                    placeholders: TemplatePlaceholders {
                        questions: vec![
                            "What's the most underrated piece of agent infrastructure?".into(),
                        ],
                        ..Default::default()
                    },
                },
            ],
        }
    }
}

impl Default for TemplatePlaceholders {
    fn default() -> Self {
        Self {
            insights: vec![],
            reminders: vec![],
            taglines: vec![],
            statements: vec![],
            punchlines: vec![],
            questions: vec![],
            topics: vec![],
            milestones: vec![],
        }
    }
}
