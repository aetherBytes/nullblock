use chrono::Utc;
use rand::seq::SliceRandom;
use rand::thread_rng;
use uuid::Uuid;

use crate::error::ContentError;
use crate::models::{ContentStatus, GenerateContentResponse};

use super::templates::{TemplateConfig, TemplatePlaceholders, TemplateVariant};

pub struct ContentGenerator {
    config: TemplateConfig,
}

impl ContentGenerator {
    pub fn new(config: TemplateConfig) -> Self {
        Self { config }
    }

    pub fn generate(
        &self,
        theme: &str,
        include_image: bool,
    ) -> Result<GenerateContentResponse, ContentError> {
        let theme_config = self
            .config
            .themes
            .iter()
            .find(|t| t.name == theme)
            .ok_or_else(|| {
                ContentError::GenerationError(format!("Unknown theme: {}", theme))
            })?;

        let mut rng = thread_rng();

        let candidates: Vec<&TemplateVariant> = if include_image {
            theme_config.variants.iter().collect()
        } else {
            theme_config
                .variants
                .iter()
                .filter(|v| !v.requires_image)
                .collect()
        };

        let candidates = if candidates.is_empty() {
            theme_config.variants.iter().collect::<Vec<_>>()
        } else {
            candidates
        };

        let variant = candidates.choose(&mut rng).ok_or_else(|| {
            ContentError::GenerationError("No template variants available".into())
        })?;

        let text = self.replace_placeholders(&variant.template, &theme_config.placeholders);

        let image_prompt = if variant.requires_image && include_image {
            Some(self.generate_image_prompt(&text))
        } else {
            None
        };

        Ok(GenerateContentResponse {
            id: Uuid::new_v4(),
            theme: theme.to_string(),
            text,
            tags: variant.tags.clone(),
            image_prompt,
            status: ContentStatus::Pending,
            created_at: Utc::now(),
        })
    }

    pub fn generate_image_prompt(&self, content: &str) -> String {
        format!(
            "Retro-futuristic propaganda poster style. Bold geometric shapes, muted industrial palette with accent colors. \
            A stylized scene depicting autonomous systems and networked infrastructure. \
            Text overlay inspiration: \"{}\". \
            Style: Soviet constructivism meets cyberpunk corporate art. No photorealism. \
            Clean lines, dramatic lighting, monolithic architecture.",
            content.chars().take(100).collect::<String>()
        )
    }

    fn replace_placeholders(&self, template: &str, placeholders: &TemplatePlaceholders) -> String {
        let mut rng = thread_rng();
        let mut result = template.to_string();

        let replacements: Vec<(&str, &[String])> = vec![
            ("{insight}", &placeholders.insights),
            ("{reminder}", &placeholders.reminders),
            ("{tagline}", &placeholders.taglines),
            ("{statement}", &placeholders.statements),
            ("{punchline}", &placeholders.punchlines),
            ("{question}", &placeholders.questions),
            ("{topic}", &placeholders.topics),
            ("{milestone}", &placeholders.milestones),
        ];

        for (placeholder, pool) in replacements {
            if result.contains(placeholder) {
                if let Some(value) = pool.choose(&mut rng) {
                    result = result.replacen(placeholder, value, 1);
                }
            }
        }

        result
    }
}
